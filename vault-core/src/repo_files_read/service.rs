use std::{collections::HashMap, pin::Pin, sync::Arc};

use futures::{
    channel::mpsc,
    io::{self, BufReader},
    AsyncRead, AsyncWrite, SinkExt, StreamExt, TryStreamExt,
};

use crate::{
    cipher::{data_cipher::decrypt_size, Cipher},
    remote_files::RemoteFilesService,
    repo_files::{
        selectors as repo_files_selectors,
        state::{RepoFile, RepoFileType},
    },
    repo_files_list::{
        errors::GetListRecursiveError, state::RepoFilesListRecursiveItem, RepoFilesListService,
    },
    repos::ReposService,
    runtime, store,
    utils::{path_utils, sender_writer::SenderWriter},
};

use super::{
    errors::GetFilesReaderError,
    mutations, selectors,
    state::{RemoteZipEntry, RepoFileReader},
};

pub struct RepoFilesReadService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    repo_files_list_service: Arc<RepoFilesListService>,
    store: Arc<store::Store>,
    runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
}

impl RepoFilesReadService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        repo_files_list_service: Arc<RepoFilesListService>,
        store: Arc<store::Store>,
        runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
    ) -> Self {
        Self {
            repos_service,
            remote_files_service,
            repo_files_list_service,
            store,
            runtime,
        }
    }

    async fn get_remote_file_reader(
        &self,
        mount_id: &str,
        remote_path: &str,
        name: &str,
        content_type: Option<&str>,
        cipher: &Cipher,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let encrypted_reader = self
            .remote_files_service
            .get_file_reader(&mount_id, &remote_path)
            .await?;

        let size = decrypt_size(encrypted_reader.size.try_into().unwrap())
            .map_err(GetFilesReaderError::DecryptSizeError)?;

        let decrypt_reader = Box::pin(cipher.decrypt_reader(encrypted_reader.reader));

        Ok(RepoFileReader {
            name: name.to_owned(),
            size: Some(size),
            content_type: content_type.map(str::to_string),
            reader: decrypt_reader,
        })
    }

    async fn get_file_reader_file(
        &self,
        file: &RepoFile,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let name = file.decrypted_name()?;

        let cipher = self.repos_service.get_cipher(&file.repo_id)?;

        self.get_remote_file_reader(
            &file.mount_id,
            &file.remote_path,
            name,
            file.content_type.as_deref(),
            &cipher,
        )
        .await
    }

    async fn create_zip<W: AsyncWrite + Unpin>(
        &self,
        writer: W,
        entries: Vec<RemoteZipEntry>,
    ) -> Result<(), std::io::Error> {
        let mut zip_writer = async_zip_futures::write::ZipFileWriter::new(writer);

        for entry in entries {
            match &entry.typ {
                RepoFileType::Dir => {
                    let zip_entry_builder = async_zip_futures::ZipEntryBuilder::new(
                        entry.filename,
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
                        entry.filename,
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
                            "",
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

                    io::copy_buf(
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

    fn get_zip_reader(
        self: Arc<Self>,
        entries: Vec<RemoteZipEntry>,
    ) -> Pin<Box<dyn AsyncRead + Send + Sync + 'static>> {
        let (tx, rx) = mpsc::channel::<std::io::Result<Vec<u8>>>(10);

        let mut error_tx = tx.clone();

        let create_zip_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            match create_zip_self
                .create_zip(SenderWriter::new(tx), entries)
                .await
            {
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
        dir_path_prefix: Option<String>,
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
                                    *relative_repo_path =
                                        path_utils::join_paths(dir_path_prefix, relative_repo_path);
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

    async fn get_dir_zip_name_entries(
        &self,
        file: &RepoFile,
    ) -> Result<(String, Vec<RemoteZipEntry>), GetFilesReaderError> {
        let zip_name = format!("{}.zip", file.decrypted_name()?);

        let remote_zip_entries = self.get_file_remote_zip_entries(file, None).await?;

        Ok((zip_name, remote_zip_entries))
    }

    async fn get_files_zip_name_entries(
        &self,
        files: &[RepoFile],
    ) -> Result<(String, Vec<RemoteZipEntry>), GetFilesReaderError> {
        let (zip_name, file_names) = self.store.with_state(|state| {
            let zip_name = selectors::select_files_zip_name(state, files);

            let file_names = files
                .iter()
                .filter_map(|file| {
                    repo_files_selectors::select_file_name(state, file)
                        .map(|name| (file.id.clone(), name.to_owned()))
                })
                .collect::<HashMap<String, String>>();

            (zip_name, file_names)
        });

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
                RepoFileType::Dir => Some(format!("/{}", file_name)),
                RepoFileType::File => None,
            };

            remote_zip_entries.extend(
                self.get_file_remote_zip_entries(file, dir_path_prefix)
                    .await?
                    .into_iter(),
            );
        }

        Ok((zip_name, remote_zip_entries))
    }

    pub async fn get_files_reader(
        self: Arc<Self>,
        files: &[RepoFile],
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let (name, remote_zip_entries) = match files.len() {
            0 => panic!("files cannot be empty"),
            1 => {
                let file = files.get(0).unwrap();

                match file.typ {
                    RepoFileType::Dir => self.get_dir_zip_name_entries(file).await?,
                    RepoFileType::File => {
                        return self.get_file_reader_file(file).await;
                    }
                }
            }
            _ => self.get_files_zip_name_entries(files).await?,
        };

        let reader = self.get_zip_reader(remote_zip_entries);

        Ok(RepoFileReader {
            name,
            size: None,
            content_type: Some(String::from("application/zip")),
            reader,
        })
    }
}
