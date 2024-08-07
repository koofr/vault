use std::{collections::HashMap, sync::Arc};

use futures::{
    channel::mpsc, io::BufReader, AsyncWrite, FutureExt, SinkExt, StreamExt, TryStreamExt,
};
use vault_crypto::data_cipher::decrypt_size;

use crate::{
    cipher::{errors::DecryptSizeError, Cipher},
    common::state::{BoxAsyncRead, SizeInfo},
    remote_files::RemoteFilesService,
    repo_files::{
        selectors as repo_files_selectors,
        state::{RepoFile, RepoFileType},
    },
    repo_files_list::{
        errors::GetListRecursiveError, state::RepoFilesListRecursiveItem, RepoFilesListService,
    },
    repo_files_tags::RepoFilesTagsService,
    repos::ReposService,
    runtime, store,
    types::{DecryptedName, DecryptedPath, MountId, RemotePath, RepoFileId},
    utils::{md5_reader, on_end_reader::OnEndReader, repo_path_utils, sender_writer::SenderWriter},
};

use super::{
    errors::GetFilesReaderError,
    mutations, selectors,
    state::{GetRemoteZipEntries, RemoteZipEntry, RepoFileReader, RepoFileReaderProvider},
};

pub struct RepoFilesReadService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    repo_files_list_service: Arc<RepoFilesListService>,
    repo_files_tags_service: Arc<RepoFilesTagsService>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,
}

impl RepoFilesReadService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        repo_files_list_service: Arc<RepoFilesListService>,
        repo_files_tags_service: Arc<RepoFilesTagsService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Self {
        Self {
            repos_service,
            remote_files_service,
            repo_files_list_service,
            repo_files_tags_service,
            store,
            runtime,
        }
    }

    async fn get_remote_file_reader(
        &self,
        mount_id: &MountId,
        remote_path: &RemotePath,
        name: DecryptedName,
        content_type: Option<&str>,
        unique_name: Option<&str>,
        cipher: &Cipher,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let encrypted_reader = self
            .remote_files_service
            .get_file_reader(&mount_id, &remote_path)
            .await?;

        let size = decrypt_size(encrypted_reader.size)
            .map_err(|err| GetFilesReaderError::DecryptSizeError(DecryptSizeError::from(err)))?;

        let decrypt_reader = Box::pin(cipher.decrypt_reader_async(encrypted_reader.reader));

        Ok(RepoFileReader {
            name,
            size: SizeInfo::Exact(size),
            content_type: content_type.map(str::to_string),
            remote_file: Some(encrypted_reader.file),
            unique_name: unique_name.map(str::to_string),
            reader: decrypt_reader,
        })
    }

    async fn get_file_reader_file(
        &self,
        file: &RepoFile,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let name = file.decrypted_name()?.to_owned();

        let cipher = self.repos_service.get_cipher(&file.repo_id)?;

        let reader = self
            .get_remote_file_reader(
                &file.mount_id,
                &file.remote_path,
                name,
                file.content_type.as_deref(),
                Some(&file.unique_name),
                &cipher,
            )
            .await?;

        if file.hash().is_none() {
            if let Some(remote_file_hash) = reader
                .remote_file
                .as_ref()
                .and_then(|file| file.hash.clone())
            {
                return Ok(reader.wrap_reader(|reader| {
                    self.generate_missing_hash_reader(file, remote_file_hash, reader)
                }));
            }
        }

        Ok(reader)
    }

    fn generate_missing_hash_reader(
        &self,
        file: &RepoFile,
        remote_file_hash: String,
        reader: BoxAsyncRead,
    ) -> BoxAsyncRead {
        let runtime = self.runtime.clone();
        let repo_files_tags_service = self.repo_files_tags_service.clone();
        let repo_id = file.repo_id.clone();
        let path = file.encrypted_path.clone();
        let (md5_reader, md5_digest_future) = md5_reader::MD5Reader::new(reader);

        Box::pin(OnEndReader::new(
            md5_reader,
            Box::new(move |res| {
                if res.is_ok() {
                    runtime.spawn(Box::pin(async move {
                        let repo_id = repo_id;
                        let path = path;

                        if let Ok(hash) = md5_digest_future.await.map(|hash| hash.to_vec()) {
                            if let Err(err) = repo_files_tags_service.set_tags_hash(
                                &repo_id,
                                &path,
                                remote_file_hash,
                                hash,
                            ).await {
                                log::warn!(
                                    "RepoFilesReadService generate_missing_hash_reader failed to set tags: {}",
                                    err,
                                );
                            }
                        }
                    }));
                }
            }),
        ))
    }

    fn get_file_reader_file_provider(
        self: Arc<Self>,
        file: RepoFile,
    ) -> Result<RepoFileReaderProvider, GetFilesReaderError> {
        let name = file.decrypted_name()?.to_owned();
        let size = file
            .decrypted_size()?
            .map(SizeInfo::Exact)
            .unwrap_or(SizeInfo::Unknown);

        let this = self.clone();
        let file = Arc::new(file);

        Ok(RepoFileReaderProvider {
            name,
            size,
            unique_name: Some(file.unique_name.clone()),
            reader_builder: Box::new(move || {
                let this = this.clone();
                let file = file.clone();

                async move { this.get_file_reader_file(&file).await }.boxed()
            }),
        })
    }

    async fn create_zip<W: AsyncWrite + Unpin>(
        &self,
        writer: W,
        entries: &[RemoteZipEntry],
    ) -> Result<(), std::io::Error> {
        let mut zip_writer = async_zip_futures::write::ZipFileWriter::new(writer);

        for entry in entries {
            match &entry.typ {
                RepoFileType::Dir => {
                    let zip_entry_builder = async_zip_futures::ZipEntryBuilder::new(
                        entry.filename.clone(),
                        async_zip_futures::Compression::Stored,
                    )
                    .last_modification_date(entry.modified)
                    .unix_permissions(0o755);

                    zip_writer
                        .write_entry_whole(zip_entry_builder, &[])
                        .await
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
                }
                RepoFileType::File => {
                    let zip_entry_builder = async_zip_futures::ZipEntryBuilder::new(
                        entry.filename.clone(),
                        async_zip_futures::Compression::Stored,
                    )
                    .last_modification_date(entry.modified)
                    .unix_permissions(0o644);

                    let cipher = self
                        .repos_service
                        .get_cipher(&entry.repo_id)
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

                    let reader = self
                        .get_remote_file_reader(
                            &entry.mount_id,
                            &entry.remote_path,
                            DecryptedName("".into()),
                            None,
                            None,
                            &cipher,
                        )
                        .await
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

                    let mut entry_writer =
                        zip_writer
                            .write_entry_stream(zip_entry_builder)
                            .await
                            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

                    futures::io::copy_buf(
                        BufReader::with_capacity(1024 * 1024, reader.reader),
                        &mut entry_writer,
                    )
                    .await?;

                    entry_writer
                        .close()
                        .await
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
                }
            }
        }

        zip_writer
            .close()
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

        Ok(())
    }

    fn get_zip_reader(self: Arc<Self>, entries: Vec<RemoteZipEntry>) -> BoxAsyncRead {
        let (tx, rx) = mpsc::channel::<std::io::Result<Vec<u8>>>(10);

        let mut error_tx = tx.clone();

        let this = self.clone();

        self.runtime.spawn(Box::pin(async move {
            match this.create_zip(SenderWriter::new(tx), &entries).await {
                Ok(_) => {}
                Err(err) => {
                    let _ = error_tx.send(Err(err)).await;
                }
            }
        }));

        Box::pin(BufReader::with_capacity(1024 * 1024, rx.into_async_read()))
    }

    async fn get_file_remote_zip_entries(
        &self,
        file: &RepoFile,
        dir_path_prefix: Option<DecryptedPath>,
    ) -> Result<Vec<RemoteZipEntry>, GetFilesReaderError> {
        match file.typ {
            RepoFileType::Dir => {
                let mut items = self
                    .repo_files_list_service
                    .get_list_recursive(&file)
                    .await
                    .map_err(|err| match err {
                        GetListRecursiveError::RepoNotFound(err) => {
                            GetFilesReaderError::RepoNotFound(err)
                        }
                        GetListRecursiveError::RepoLocked(err) => {
                            GetFilesReaderError::RepoLocked(err)
                        }
                        GetListRecursiveError::DecryptFilenameError(err) => {
                            GetFilesReaderError::DecryptFilenameError(err)
                        }
                        GetListRecursiveError::RemoteError(err) => {
                            GetFilesReaderError::RemoteError(err)
                        }
                    })?
                    .collect::<Vec<RepoFilesListRecursiveItem>>()
                    .await;

                if let Some(dir_path_prefix) = &dir_path_prefix {
                    for mut item in &mut items {
                        match &mut item {
                            RepoFilesListRecursiveItem::File {
                                ref mut relative_repo_path,
                                ..
                            } => match relative_repo_path {
                                Ok(relative_repo_path) => {
                                    *relative_repo_path = repo_path_utils::join_paths(
                                        dir_path_prefix,
                                        relative_repo_path,
                                    );
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }

                Ok(mutations::list_recursive_items_to_remote_zip_entries(
                    items,
                )?)
            }
            RepoFileType::File => Ok(vec![mutations::file_to_remote_zip_entry(file)?]),
        }
    }

    fn get_dir_zip_name_entries(
        self: Arc<Self>,
        file: RepoFile,
    ) -> Result<(DecryptedName, GetRemoteZipEntries), GetFilesReaderError> {
        let zip_name = DecryptedName(format!("{}.zip", file.decrypted_name()?.0));

        let this = self.clone();
        let file = Arc::new(file);

        let get_remote_zip_entries: GetRemoteZipEntries = Box::new(move || {
            let this = this.clone();
            let file = file.clone();

            async move { this.get_file_remote_zip_entries(&file, None).await }.boxed()
        });

        Ok((zip_name, get_remote_zip_entries))
    }

    fn get_files_zip_name_entries(
        self: Arc<Self>,
        files: Vec<RepoFile>,
    ) -> Result<(DecryptedName, GetRemoteZipEntries), GetFilesReaderError> {
        let (zip_name, file_names) = self.store.with_state(|state| {
            let zip_name = selectors::select_files_zip_name(state, &files);

            let file_names = files
                .iter()
                .filter_map(|file| {
                    repo_files_selectors::select_file_name(state, file)
                        .map(|name| (file.id.clone(), name.to_owned()))
                        .ok()
                })
                .collect::<HashMap<RepoFileId, DecryptedName>>();

            (zip_name, file_names)
        });

        let this = self.clone();
        let files = Arc::new(files);
        let file_names = Arc::new(file_names);

        let get_remote_zip_entries: GetRemoteZipEntries = Box::new(move || {
            let this = this.clone();
            let files = files.clone();
            let file_names = file_names.clone();

            async move { this.get_files_zip_entries(&files, &file_names).await }.boxed()
        });

        Ok((zip_name, get_remote_zip_entries))
    }

    async fn get_files_zip_entries(
        &self,
        files: &[RepoFile],
        file_names: &HashMap<RepoFileId, DecryptedName>,
    ) -> Result<Vec<RemoteZipEntry>, GetFilesReaderError> {
        let mut remote_zip_entries = Vec::new();

        for file in files {
            let file_name = match file_names.get(&file.id).cloned() {
                Some(file_name) => file_name,
                None => {
                    // skip invalid files
                    continue;
                }
            };

            let dir_path_prefix = match file.typ {
                RepoFileType::Dir => Some(DecryptedPath(format!("/{}", file_name.0))),
                RepoFileType::File => None,
            };

            remote_zip_entries.extend(
                self.get_file_remote_zip_entries(file, dir_path_prefix)
                    .await?
                    .into_iter(),
            );
        }

        Ok(remote_zip_entries)
    }

    pub fn get_files_reader(
        self: Arc<Self>,
        files: Vec<RepoFile>,
    ) -> Result<RepoFileReaderProvider, GetFilesReaderError> {
        let (name, get_remote_zip_entries) = match files.len() {
            0 => return Err(GetFilesReaderError::FilesEmpty),
            1 => {
                let file = files.into_iter().next().unwrap();

                match file.typ {
                    RepoFileType::Dir => self.clone().get_dir_zip_name_entries(file)?,
                    RepoFileType::File => {
                        return self.clone().get_file_reader_file_provider(file);
                    }
                }
            }
            _ => self.clone().get_files_zip_name_entries(files)?,
        };

        let this = self.clone();
        let reader_builder_name = Arc::new(name.clone());
        let get_remote_zip_entries = Arc::new(get_remote_zip_entries);

        Ok(RepoFileReaderProvider {
            name,
            size: SizeInfo::Unknown,
            unique_name: None,
            reader_builder: Box::new(move || {
                let this = this.clone();
                let get_remote_zip_entries = get_remote_zip_entries.clone();
                let name = (*reader_builder_name).clone();

                async move {
                    let remote_zip_entries = get_remote_zip_entries().await?;

                    let size =
                        SizeInfo::Estimate(mutations::zip_size_estimate(&remote_zip_entries));
                    let reader = this.clone().get_zip_reader(remote_zip_entries);

                    Ok(RepoFileReader {
                        name,
                        size,
                        content_type: Some("application/zip".into()),
                        remote_file: None,
                        unique_name: None,
                        reader,
                    })
                }
                .boxed()
            }),
        })
    }
}
