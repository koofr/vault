use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{
    future::{BoxFuture, Shared},
    io::Cursor,
    FutureExt,
};

use crate::{
    cipher::{
        data_cipher::{decrypt_on_progress, encrypted_size},
        Cipher,
    },
    common::state::BoxAsyncRead,
    dialogs, remote,
    remote_files::{state::RemoteFilesLocation, RemoteFilesService},
    repo_files_read::{
        errors::GetFilesReaderError, state::RepoFileReaderProvider, RepoFilesReadService,
    },
    repos::{errors::RepoLockedError, ReposService},
    store,
    types::{
        DecryptedName, DecryptedPath, EncryptedName, MountId, RemoteName, RemotePath, RepoFileId,
        RepoId,
    },
    utils::{name_utils, repo_path_utils},
};

use super::{
    errors::{
        CopyFileError, CreateDirError, CreateFileError, DeleteFileError, EnsureDirError,
        GetRepoMountPathError, LoadFileError, LoadFilesError, MoveFileError, RenameFileError,
        RepoFilesErrors, UploadFileReaderError,
    },
    mutations, selectors,
    state::{RepoFileType, RepoFilesUploadConflictResolution, RepoFilesUploadResult},
};

pub struct RepoFilesService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    dialogs_service: Arc<dialogs::DialogsService>,
    store: Arc<store::Store>,
    ensure_dirs_futures:
        Arc<Mutex<HashMap<RepoFileId, Shared<BoxFuture<'static, Result<(), EnsureDirError>>>>>>,
    remote_files_mutation_subscription_id: u32,
    repos_mutation_subscription_id: u32,
}

impl RepoFilesService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        dialogs_service: Arc<dialogs::DialogsService>,
        store: Arc<store::Store>,
    ) -> Self {
        let remote_files_mutation_subscription_id = store.get_next_id();
        let remote_files_mutation_repos_service = repos_service.clone();

        let repos_mutation_subscription_id = store.get_next_id();
        let repos_mutation_repos_service = repos_service.clone();

        let repo_files_service = Self {
            repos_service,
            remote_files_service,
            repo_files_read_service,
            dialogs_service,
            store: store.clone(),
            ensure_dirs_futures: Arc::new(Mutex::new(HashMap::new())),
            remote_files_mutation_subscription_id,
            repos_mutation_subscription_id,
        };

        store.mutation_on(
            remote_files_mutation_subscription_id,
            &[store::MutationEvent::RemoteFiles],
            Box::new(move |state, notify, mutation_state, mutation_notify| {
                mutations::handle_remote_files_mutation(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    &remote_files_mutation_repos_service.get_ciphers(),
                );
            }),
        );

        store.mutation_on(
            repos_mutation_subscription_id,
            &[store::MutationEvent::Repos],
            Box::new(move |state, notify, mutation_state, mutation_notify| {
                mutations::handle_repos_mutation(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    &repos_mutation_repos_service.get_ciphers(),
                );
            }),
        );

        repo_files_service
    }

    pub fn get_repo_mount_path_cipher(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
        cipher: &Cipher,
    ) -> Result<(MountId, RemotePath), GetRepoMountPathError> {
        self.store.with_state(|state| {
            selectors::select_repo_path_to_mount_path(state, repo_id, path, cipher)
                .map_err(GetRepoMountPathError::RepoNotFound)
        })
    }

    pub fn get_repo_mount_path(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(MountId, RemotePath), GetRepoMountPathError> {
        let cipher = self
            .repos_service
            .get_cipher(repo_id)
            .map_err(GetRepoMountPathError::RepoLocked)?;

        self.get_repo_mount_path_cipher(repo_id, path, &cipher)
    }

    pub fn get_repo_remote_location(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<RemoteFilesLocation, GetRepoMountPathError> {
        self.get_repo_mount_path(repo_id, path)
            .map(|(mount_id, path)| RemoteFilesLocation { mount_id, path })
    }

    pub async fn load_files(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(), LoadFilesError> {
        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoNotFound(err) => LoadFilesError::RepoNotFound(err),
                    GetRepoMountPathError::RepoLocked(err) => LoadFilesError::RepoLocked(err),
                })?;

        self.remote_files_service
            .load_files(&mount_id, &remote_path)
            .await
            .map_err(LoadFilesError::RemoteError)?;

        Ok(())
    }

    pub async fn load_file(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(), LoadFileError> {
        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoNotFound(err) => LoadFileError::RepoNotFound(err),
                    GetRepoMountPathError::RepoLocked(err) => LoadFileError::RepoLocked(err),
                })?;

        self.remote_files_service
            .load_file(&mount_id, &remote_path)
            .await
            .map_err(LoadFileError::RemoteError)?;

        Ok(())
    }

    pub fn encrypt_filename(
        &self,
        repo_id: &RepoId,
        name: &DecryptedName,
    ) -> Result<EncryptedName, RepoLockedError> {
        let cipher = self.repos_service.get_cipher(&repo_id)?;

        Ok(cipher.encrypt_filename(name))
    }

    pub fn get_file_reader(
        self: Arc<Self>,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<RepoFileReaderProvider, GetFilesReaderError> {
        let file = self
            .store
            .with_state(|state| {
                selectors::select_file(state, &selectors::get_file_id(repo_id, path))
                    .map(|file| file.clone())
            })
            .ok_or(GetFilesReaderError::FileNotFound)?;

        self.repo_files_read_service
            .clone()
            .get_files_reader(vec![file])
    }

    pub async fn upload_file_reader(
        self: Arc<Self>,
        repo_id: &RepoId,
        parent_path: &DecryptedPath,
        name: &DecryptedName,
        reader: BoxAsyncRead,
        size: Option<i64>,
        conflict_resolution: RepoFilesUploadConflictResolution,
        on_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
    ) -> Result<RepoFilesUploadResult, UploadFileReaderError> {
        self.clone().ensure_dirs(repo_id, parent_path).await?;

        let cipher = self.repos_service.get_cipher(&repo_id)?;

        let (mount_id, remote_parent_path) = self
            .store
            .with_state(|state| {
                selectors::select_repo_path_to_mount_path(state, repo_id, parent_path, &cipher)
            })
            .map_err(UploadFileReaderError::RepoNotFound)?;

        let encrypted_size = size.map(encrypted_size);
        let encrypted_name = cipher.encrypt_filename(name);
        let encrypted_reader = cipher.encrypt_reader(reader);

        let (_, remote_file) = self
            .remote_files_service
            .upload_file_reader(
                &mount_id,
                &remote_parent_path,
                &RemoteName(encrypted_name.0),
                Box::pin(encrypted_reader),
                encrypted_size,
                conflict_resolution.into(),
                on_progress.map(decrypt_on_progress),
            )
            .await
            .map_err(UploadFileReaderError::RemoteError)?;

        let name = cipher.decrypt_filename(&remote_file.name)?;
        let path = repo_path_utils::join_path_name(parent_path, &name);
        let file_id = selectors::get_file_id(repo_id, &path);

        Ok(RepoFilesUploadResult {
            file_id,
            name,
            remote_file,
        })
    }

    pub async fn delete_files(
        &self,
        files: &[(RepoId, DecryptedPath)],
        before_delete: Option<Box<dyn Fn() + Send + 'static>>,
    ) -> Result<(), DeleteFileError> {
        if self
            .dialogs_service
            .show(dialogs::state::DialogShowOptions {
                title: String::from("Delete files"),
                message: Some(if files.len() == 1 {
                    String::from("Do you really want to delete 1 item?")
                } else {
                    format!("Do you really want to delete {} items?", files.len())
                }),
                confirm_button_text: String::from("Delete"),
                cancel_button_text: Some(String::from("Cancel")),
                ..self.dialogs_service.build_confirm()
            })
            .await
            .is_some()
        {
            if let Some(before_delete) = before_delete {
                before_delete();
            }

            for (repo_id, path) in files {
                let (mount_id, remote_path) =
                    self.get_repo_mount_path(repo_id, path)
                        .map_err(|e| match e {
                            GetRepoMountPathError::RepoLocked(err) => {
                                DeleteFileError::RepoLocked(err)
                            }
                            GetRepoMountPathError::RepoNotFound(err) => {
                                DeleteFileError::RepoNotFound(err)
                            }
                        })?;

                self.remote_files_service
                    .delete_file(&mount_id, &remote_path)
                    .await
                    .map_err(DeleteFileError::RemoteError)?;
            }
        } else {
            return Err(DeleteFileError::Canceled);
        }

        Ok(())
    }

    pub async fn create_dir(
        &self,
        repo_id: &RepoId,
        parent_path: &DecryptedPath,
    ) -> Result<(DecryptedName, DecryptedPath), CreateDirError> {
        let input_value_validator_store = self.store.clone();
        let input_value_validator_repo_id = repo_id.to_owned();
        let input_value_validator_parent_path = parent_path.to_owned();

        let name = match self
            .dialogs_service
            .show_validator(
                dialogs::state::DialogShowOptions {
                    input_placeholder: Some(String::from("Folder name")),
                    confirm_button_text: String::from("Create folder"),
                    ..self
                        .dialogs_service
                        .build_prompt(String::from("Enter new folder name"))
                },
                move |value| {
                    input_value_validator_store.with_state(|state| {
                        selectors::select_check_new_name_valid(
                            state,
                            &input_value_validator_repo_id,
                            &input_value_validator_parent_path,
                            &DecryptedName(value.clone()),
                        )
                        .map(|_| value)
                    })
                },
            )
            .await
        {
            Some(name) => DecryptedName(name?),
            None => return Err(CreateDirError::Canceled),
        };

        let path = repo_path_utils::join_path_name(parent_path, &name);

        self.create_dir_name(repo_id, parent_path, &name).await?;

        Ok((name, path))
    }

    pub async fn create_dir_name(
        &self,
        repo_id: &RepoId,
        parent_path: &DecryptedPath,
        name: &DecryptedName,
    ) -> Result<(), CreateDirError> {
        let (mount_id, remote_parent_path) = self
            .get_repo_mount_path(repo_id, parent_path)
            .map_err(|e| match e {
                GetRepoMountPathError::RepoLocked(err) => CreateDirError::RepoLocked(err),
                GetRepoMountPathError::RepoNotFound(err) => CreateDirError::RepoNotFound(err),
            })?;

        let encrypted_name = self.encrypt_filename(repo_id, name)?;

        self.remote_files_service
            .create_dir_name(&mount_id, &remote_parent_path, RemoteName(encrypted_name.0))
            .await
            .map_err(CreateDirError::RemoteError)?;

        Ok(())
    }

    pub async fn ensure_dir(
        &self,
        repo_id: RepoId,
        path: DecryptedPath,
    ) -> Result<(), EnsureDirError> {
        let (parent_path, name) = match repo_path_utils::split_parent_name(&path) {
            Some(val) => val,
            None => {
                return Ok(());
            }
        };

        let file_id = selectors::get_file_id(&repo_id, &path);

        match self.store.with_state(|state| {
            selectors::select_file(state, &file_id).map(|file| file.typ.clone())
        }) {
            Some(RepoFileType::File) => {
                Err(EnsureDirError::RemoteError(RepoFilesErrors::not_a_dir()))
            }
            Some(RepoFileType::Dir) => Ok(()),
            None => match self.ensure_dir_load_file(&repo_id, &path).await {
                Ok(()) => Ok(()),
                Err(EnsureDirError::RemoteError(remote::RemoteError::ApiError {
                    code: remote::ApiErrorCode::NotFound,
                    ..
                })) => match self.create_dir_name(&repo_id, &parent_path, &name).await {
                    Ok(()) => Ok(()),
                    Err(CreateDirError::RemoteError(remote::RemoteError::ApiError {
                        code: remote::ApiErrorCode::AlreadyExists,
                        ..
                    })) => self.ensure_dir_load_file(&repo_id, &path).await,
                    Err(err) => Err(err.into()),
                },
                Err(err) => Err(err),
            },
        }
    }

    async fn ensure_dir_load_file(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(), EnsureDirError> {
        Ok(self.load_file(repo_id, path).await?)
    }

    pub async fn ensure_dirs(
        self: Arc<Self>,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(), EnsureDirError> {
        for path in repo_path_utils::paths_chain(&path) {
            if path.is_root() {
                continue;
            }

            self.clone().ensure_dir_synchronized(repo_id, &path).await?;
        }

        Ok(())
    }

    pub async fn ensure_dir_synchronized(
        self: Arc<Self>,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(), EnsureDirError> {
        let file_id = selectors::get_file_id(&repo_id, &path);

        let ensure_future = self
            .ensure_dirs_futures
            .lock()
            .unwrap()
            .get(&file_id)
            .map(|f| f.clone());

        match ensure_future {
            Some(ensure_future) => ensure_future.await,
            None => {
                let repo_id = repo_id.to_owned();
                let path = path.to_owned();
                let ensure_dir_self = self.clone();
                let ensure_future = async move { ensure_dir_self.ensure_dir(repo_id, path).await }
                    .boxed()
                    .shared();

                self.ensure_dirs_futures
                    .lock()
                    .unwrap()
                    .insert(file_id.clone(), ensure_future.clone());

                let res = ensure_future.await;

                self.ensure_dirs_futures.lock().unwrap().remove(&file_id);

                res
            }
        }
    }

    pub async fn create_file(
        self: Arc<Self>,
        repo_id: &RepoId,
        parent_path: &DecryptedPath,
        name: &str,
    ) -> Result<(DecryptedName, DecryptedPath), CreateFileError> {
        let input_value_validator_store = self.store.clone();
        let input_value_validator_repo_id = repo_id.to_owned();
        let input_value_validator_parent_path = parent_path.to_owned();
        let input_value_selected = Some(name_utils::split_name_ext(&name).0.to_owned());

        let name = match self
            .dialogs_service
            .show_validator(
                dialogs::state::DialogShowOptions {
                    input_value: name.to_owned(),
                    input_value_selected,
                    input_placeholder: Some(String::from("File name")),
                    confirm_button_text: String::from("Create file"),
                    ..self
                        .dialogs_service
                        .build_prompt(String::from("Enter new file name"))
                },
                move |value| {
                    input_value_validator_store.with_state(|state| {
                        selectors::select_check_new_name_valid(
                            state,
                            &input_value_validator_repo_id,
                            &input_value_validator_parent_path,
                            &DecryptedName(value.clone()),
                        )
                        .map(|_| value)
                    })
                },
            )
            .await
        {
            Some(name) => DecryptedName(name?),
            None => return Err(CreateFileError::Canceled),
        };

        let path = repo_path_utils::join_path_name(&parent_path, &name);

        self.create_file_name(repo_id, parent_path, &name).await?;

        Ok((name, path))
    }

    pub async fn create_file_name(
        self: Arc<Self>,
        repo_id: &RepoId,
        parent_path: &DecryptedPath,
        name: &DecryptedName,
    ) -> Result<(), CreateFileError> {
        self.upload_file_reader(
            &repo_id,
            &parent_path,
            &name,
            Box::pin(Cursor::new(vec![])),
            Some(0),
            RepoFilesUploadConflictResolution::Error,
            None,
        )
        .await?;

        Ok(())
    }

    pub async fn rename_file(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
    ) -> Result<(), RenameFileError> {
        let (mount_id, remote_path, original_name, typ) = match self.store.with_state(|state| {
            selectors::select_file(state, &selectors::get_file_id(repo_id, path)).map(|file| {
                (
                    file.mount_id.clone(),
                    file.remote_path.clone(),
                    file.decrypted_name().map(ToOwned::to_owned),
                    file.typ.clone(),
                )
            })
        }) {
            Some(x) => x,
            None => return Err(RenameFileError::RemoteError(RepoFilesErrors::not_a_dir())),
        };
        let original_name = original_name?;

        let input_value = original_name.0.clone();
        let input_value_selected = Some(match typ {
            RepoFileType::Dir => input_value.clone(),
            RepoFileType::File => name_utils::split_name_ext(&input_value).0.to_owned(),
        });

        let input_value_validator_store = self.store.clone();
        let input_value_validator_repo_id = repo_id.to_owned();
        let input_value_validator_path = path.to_owned();

        if let Some(name) = self
            .dialogs_service
            .show_validator(
                dialogs::state::DialogShowOptions {
                    input_value,
                    input_value_selected,
                    input_placeholder: Some(String::from("New name")),
                    confirm_button_text: String::from("Rename"),
                    ..self
                        .dialogs_service
                        .build_prompt(format!("Enter new name for '{}'", original_name.0))
                },
                move |value| {
                    input_value_validator_store.with_state(|state| {
                        selectors::select_check_rename_file(
                            state,
                            &input_value_validator_repo_id,
                            &input_value_validator_path,
                            &DecryptedName(value.clone()),
                        )
                        .map(|_| value)
                    })
                },
            )
            .await
        {
            let name = DecryptedName(name?);
            let encrypted_name = self.encrypt_filename(&repo_id, &name)?;

            self.remote_files_service
                .rename_file(&mount_id, &remote_path, RemoteName(encrypted_name.0))
                .await
                .map_err(RenameFileError::RemoteError)?;
        }
        Ok(())
    }

    pub async fn copy_file(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
        to_parent_path: &DecryptedPath,
    ) -> Result<(), CopyFileError> {
        let name = repo_path_utils::path_to_name(path).ok_or(CopyFileError::InvalidPath)?;

        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoLocked(err) => CopyFileError::RepoLocked(err),
                    GetRepoMountPathError::RepoNotFound(err) => CopyFileError::RepoNotFound(err),
                })?;

        let (to_mount_id, to_remote_path) = self
            .get_repo_mount_path(
                repo_id,
                &repo_path_utils::join_path_name(to_parent_path, &name),
            )
            .map_err(|e| match e {
                GetRepoMountPathError::RepoLocked(err) => CopyFileError::RepoLocked(err),
                GetRepoMountPathError::RepoNotFound(err) => CopyFileError::RepoNotFound(err),
            })?;

        self.remote_files_service
            .copy_file(&mount_id, &remote_path, &to_mount_id, &to_remote_path)
            .await
            .map_err(CopyFileError::RemoteError)
    }

    pub async fn move_file(
        &self,
        repo_id: &RepoId,
        path: &DecryptedPath,
        to_parent_path: &DecryptedPath,
    ) -> Result<(), MoveFileError> {
        let name = repo_path_utils::path_to_name(path).ok_or(MoveFileError::InvalidPath)?;

        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoLocked(err) => MoveFileError::RepoLocked(err),
                    GetRepoMountPathError::RepoNotFound(err) => MoveFileError::RepoNotFound(err),
                })?;

        let (to_mount_id, to_path) = self
            .get_repo_mount_path(
                repo_id,
                &repo_path_utils::join_path_name(to_parent_path, &name),
            )
            .map_err(|e| match e {
                GetRepoMountPathError::RepoLocked(err) => MoveFileError::RepoLocked(err),
                GetRepoMountPathError::RepoNotFound(err) => MoveFileError::RepoNotFound(err),
            })?;

        self.remote_files_service
            .move_file(&mount_id, &remote_path, &to_mount_id, &to_path)
            .await
            .map_err(MoveFileError::RemoteError)
    }

    pub async fn get_unused_name(
        &self,
        repo_id: &RepoId,
        parent_path: &DecryptedPath,
        name: &DecryptedName,
    ) -> Result<DecryptedName, LoadFilesError> {
        self.load_files(repo_id, parent_path).await?;

        Ok(self.store.with_state(|state| {
            let used_names = selectors::select_used_names(state, repo_id, parent_path);

            selectors::get_unused_name(used_names, name)
        }))
    }
}

impl Drop for RepoFilesService {
    fn drop(&mut self) {
        self.store
            .mutation_remove_listener(self.remote_files_mutation_subscription_id);
        self.store
            .mutation_remove_listener(self.repos_mutation_subscription_id);
    }
}
