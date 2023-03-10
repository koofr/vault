use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use futures::{
    future::{BoxFuture, Shared},
    AsyncRead, FutureExt,
};

use crate::{
    cipher::{
        data_cipher::{decrypt_on_progress, encrypted_size},
        Cipher,
    },
    http,
    remote::{self, models},
    remote_files::{state::RemoteFilesLocation, RemoteFilesService},
    repo_files_read::{errors::GetFilesReaderError, state::RepoFileReader, RepoFilesReadService},
    repos::{errors::RepoLockedError, ReposService},
    store,
    utils::path_utils,
};

use super::{
    errors::{
        CopyFileError, CreateDirError, DecryptFilesError, DeleteFileError, EnsureDirError,
        GetRepoMountPathError, LoadFileError, LoadFilesError, MoveFileError, RenameFileError,
        RepoFilesErrors, RepoMountPathToPathError, UploadFileReaderError,
    },
    mutations, selectors,
    state::{RepoFileType, RepoFilesUploadConflictResolution, RepoFilesUploadResult},
};

pub struct RepoFilesService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    store: Arc<store::Store>,
    ensure_dirs_futures:
        Arc<Mutex<HashMap<String, Shared<BoxFuture<'static, Result<(), EnsureDirError>>>>>>,
}

impl RepoFilesService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            repos_service,
            remote_files_service,
            repo_files_read_service,
            store,
            ensure_dirs_futures: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_repo_mount_path_cipher(
        &self,
        repo_id: &str,
        path: &str,
        cipher: &Cipher,
    ) -> Result<(String, String), GetRepoMountPathError> {
        self.store.with_state(|state| {
            selectors::select_repo_path_to_mount_path(state, repo_id, path, cipher)
                .map_err(GetRepoMountPathError::RepoNotFound)
        })
    }

    pub fn get_repo_mount_path(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<(String, String), GetRepoMountPathError> {
        let cipher = self
            .repos_service
            .get_cipher(repo_id)
            .map_err(GetRepoMountPathError::RepoLocked)?;

        self.get_repo_mount_path_cipher(repo_id, path, &cipher)
    }

    pub fn get_repo_remote_location(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<RemoteFilesLocation, GetRepoMountPathError> {
        self.get_repo_mount_path(repo_id, path)
            .map(|(mount_id, path)| RemoteFilesLocation { mount_id, path })
    }

    pub async fn load_files(&self, repo_id: &str, path: &str) -> Result<(), LoadFilesError> {
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

        self.decrypt_files(repo_id, path).map_err(|e| match e {
            DecryptFilesError::RepoNotFound(err) => LoadFilesError::RepoNotFound(err),
            DecryptFilesError::RepoLocked(err) => LoadFilesError::RepoLocked(err),
        })?;

        Ok(())
    }

    pub async fn load_file(&self, repo_id: &str, path: &str) -> Result<(), LoadFileError> {
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

        if let Some(parent_path) = path_utils::parent_path(&path) {
            let _ = self.decrypt_files(&repo_id, parent_path);
        }

        Ok(())
    }

    pub fn encrypt_filename(&self, repo_id: &str, name: &str) -> Result<String, RepoLockedError> {
        let cipher = self.repos_service.get_cipher(&repo_id)?;

        Ok(cipher.encrypt_filename(name))
    }

    pub fn decrypt_files(&self, repo_id: &str, path: &str) -> Result<(), DecryptFilesError> {
        let cipher = self.repos_service.get_cipher(repo_id)?;

        self.store.mutate(store::Event::RepoFiles, |state| {
            mutations::decrypt_files(state, repo_id, path, &cipher)
        })
    }

    pub async fn get_file_reader(
        self: Arc<Self>,
        file_id: &str,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let file = self
            .store
            .with_state(|state| selectors::select_file(state, file_id).map(|file| file.clone()))
            .ok_or(GetFilesReaderError::FileNotFound)?;

        self.repo_files_read_service
            .clone()
            .get_files_reader(&[file])
            .await
    }

    pub async fn upload_file_reader(
        self: Arc<Self>,
        repo_id: &str,
        parent_path: &str,
        name: &str,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
        size: Option<i64>,
        conflict_resolution: RepoFilesUploadConflictResolution,
        on_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
        abort: http::HttpRequestAbort,
    ) -> Result<RepoFilesUploadResult, UploadFileReaderError> {
        self.clone()
            .ensure_dirs(repo_id, parent_path)
            .await
            .map_err(|e| match e {
                EnsureDirError::RepoNotFound(err) => UploadFileReaderError::RepoNotFound(err),
                EnsureDirError::RepoLocked(err) => UploadFileReaderError::RepoLocked(err),
                EnsureDirError::DecryptFilenameError(err) => {
                    UploadFileReaderError::DecryptFilenameError(err)
                }
                EnsureDirError::RemoteError(err) => UploadFileReaderError::RemoteError(err),
            })?;

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

        let (_, remote_name) = self
            .remote_files_service
            .upload_file_reader(
                &mount_id,
                &remote_parent_path,
                &encrypted_name,
                Box::pin(encrypted_reader),
                encrypted_size,
                conflict_resolution.into(),
                on_progress.map(decrypt_on_progress),
                abort,
            )
            .await
            .map_err(UploadFileReaderError::RemoteError)?;

        let _ = self.decrypt_files(&repo_id, &parent_path);

        let name = cipher.decrypt_filename(&remote_name)?;
        let path = path_utils::join_path_name(parent_path, &name);
        let file_id = selectors::get_file_id(repo_id, &path);

        Ok(RepoFilesUploadResult { file_id, name })
    }

    pub async fn delete_file(&self, repo_id: &str, path: &str) -> Result<(), DeleteFileError> {
        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoLocked(err) => DeleteFileError::RepoLocked(err),
                    GetRepoMountPathError::RepoNotFound(err) => DeleteFileError::RepoNotFound(err),
                })?;

        self.remote_files_service
            .delete_file(&mount_id, &remote_path)
            .await
            .map_err(DeleteFileError::RemoteError)?;

        if let Some(parent_path) = path_utils::parent_path(&path) {
            let _ = self.decrypt_files(&repo_id, parent_path);
        }

        Ok(())
    }

    pub async fn create_dir(
        &self,
        repo_id: &str,
        parent_path: &str,
        name: &str,
    ) -> Result<(), CreateDirError> {
        let (mount_id, remote_parent_path) = self
            .get_repo_mount_path(repo_id, parent_path)
            .map_err(|e| match e {
                GetRepoMountPathError::RepoLocked(err) => CreateDirError::RepoLocked(err),
                GetRepoMountPathError::RepoNotFound(err) => CreateDirError::RepoNotFound(err),
            })?;

        let encrypted_name = self.encrypt_filename(repo_id, name)?;

        self.remote_files_service
            .create_dir(&mount_id, &remote_parent_path, &encrypted_name)
            .await
            .map_err(CreateDirError::RemoteError)?;

        let _ = self.decrypt_files(&repo_id, &parent_path);

        Ok(())
    }

    pub async fn ensure_dir(&self, repo_id: String, path: String) -> Result<(), EnsureDirError> {
        let (parent_path, name) = match path_utils::split_parent_name(&path) {
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
                })) => match self.create_dir(&repo_id, parent_path, name).await {
                    Ok(()) => Ok(()),
                    Err(CreateDirError::RepoLocked(err)) => Err(EnsureDirError::RepoLocked(err)),
                    Err(CreateDirError::RepoNotFound(err)) => {
                        Err(EnsureDirError::RepoNotFound(err))
                    }
                    Err(CreateDirError::RemoteError(remote::RemoteError::ApiError {
                        code: remote::ApiErrorCode::AlreadyExists,
                        ..
                    })) => self.ensure_dir_load_file(&repo_id, &path).await,
                    Err(CreateDirError::DecryptFilenameError(err)) => {
                        Err(EnsureDirError::DecryptFilenameError(err))
                    }
                    Err(CreateDirError::RemoteError(err)) => Err(EnsureDirError::RemoteError(err)),
                },
                Err(err) => Err(err),
            },
        }
    }

    async fn ensure_dir_load_file(&self, repo_id: &str, path: &str) -> Result<(), EnsureDirError> {
        match self.load_file(repo_id, path).await {
            Ok(()) => Ok(()),
            Err(LoadFileError::RepoLocked(err)) => Err(EnsureDirError::RepoLocked(err)),
            Err(LoadFileError::RepoNotFound(err)) => Err(EnsureDirError::RepoNotFound(err)),
            Err(LoadFileError::RemoteError(err)) => Err(EnsureDirError::RemoteError(err)),
        }
    }

    pub async fn ensure_dirs(
        self: Arc<Self>,
        repo_id: &str,
        path: &str,
    ) -> Result<(), EnsureDirError> {
        for path in path_utils::paths_chain(&path) {
            if path == "/" {
                continue;
            }

            self.clone().ensure_dir_synchronized(repo_id, &path).await?;
        }

        Ok(())
    }

    pub async fn ensure_dir_synchronized(
        self: Arc<Self>,
        repo_id: &str,
        path: &str,
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

    pub fn check_rename_file(
        &self,
        repo_id: &str,
        path: &str,
        name: &str,
    ) -> Result<(), RenameFileError> {
        self.store
            .with_state(|state| selectors::select_check_rename_file(state, repo_id, path, name))
    }

    pub async fn rename_file(
        &self,
        repo_id: &str,
        path: &str,
        name: &str,
    ) -> Result<(), RenameFileError> {
        self.check_rename_file(repo_id, path, name)?;

        let (mount_id, remote_path) =
            self.get_repo_mount_path(&repo_id, &path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoLocked(err) => RenameFileError::RepoLocked(err),
                    GetRepoMountPathError::RepoNotFound(err) => RenameFileError::RepoNotFound(err),
                })?;

        let encrypted_name = self.encrypt_filename(&repo_id, name)?;

        self.remote_files_service
            .rename_file(&mount_id, &remote_path, &encrypted_name)
            .await
            .map_err(RenameFileError::RemoteError)
    }

    pub async fn copy_file(
        &self,
        repo_id: &str,
        path: &str,
        to_parent_path: &str,
    ) -> Result<(), CopyFileError> {
        let name = path_utils::path_to_name(path).ok_or(CopyFileError::InvalidPath)?;

        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoLocked(err) => CopyFileError::RepoLocked(err),
                    GetRepoMountPathError::RepoNotFound(err) => CopyFileError::RepoNotFound(err),
                })?;

        let (to_mount_id, to_remote_path) = self
            .get_repo_mount_path(repo_id, &path_utils::join_path_name(to_parent_path, name))
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
        repo_id: &str,
        path: &str,
        to_parent_path: &str,
    ) -> Result<(), MoveFileError> {
        let name = path_utils::path_to_name(path).ok_or(MoveFileError::InvalidPath)?;

        let (mount_id, remote_path) =
            self.get_repo_mount_path(repo_id, path)
                .map_err(|e| match e {
                    GetRepoMountPathError::RepoLocked(err) => MoveFileError::RepoLocked(err),
                    GetRepoMountPathError::RepoNotFound(err) => MoveFileError::RepoNotFound(err),
                })?;

        let (to_mount_id, to_path) = self
            .get_repo_mount_path(repo_id, &path_utils::join_path_name(to_parent_path, &name))
            .map_err(|e| match e {
                GetRepoMountPathError::RepoLocked(err) => MoveFileError::RepoLocked(err),
                GetRepoMountPathError::RepoNotFound(err) => MoveFileError::RepoNotFound(err),
            })?;

        self.remote_files_service
            .move_file(&mount_id, &remote_path, &to_mount_id, &to_path)
            .await
            .map_err(MoveFileError::RemoteError)
    }

    pub fn mount_path_decrypt_files(
        &self,
        mount_id: &str,
        path: &str,
    ) -> Result<(), DecryptFilesError> {
        if let Some((repo_id, path)) = self.store.with_state(|state| {
            let repo_id = selectors::select_mount_path_to_repo_id(&state, &mount_id, path)?;
            let cipher = match self.repos_service.get_cipher(&repo_id) {
                Ok(cipher) => cipher,
                Err(RepoLockedError) => return None,
            };
            let (_, path) =
                match selectors::select_repo_mount_path_to_path(state, &repo_id, &path, &cipher) {
                    Ok(r) => r,
                    Err(RepoMountPathToPathError::RepoNotFound(_)) => return None,
                    Err(RepoMountPathToPathError::DecryptFilenameError(_)) => return None,
                };
            Some((repo_id.to_owned(), path.to_owned()))
        }) {
            self.decrypt_files(&repo_id, &path)?;
        }

        Ok(())
    }

    pub fn remote_file_created(&self, mount_id: &str, path: &str, file: models::FilesFile) {
        self.remote_files_service.file_created(mount_id, path, file);

        if let Some(parent_path) = path_utils::parent_path(path) {
            let _ = self.mount_path_decrypt_files(mount_id, parent_path);
        }
    }

    pub fn remote_file_removed(&self, mount_id: &str, path: &str) {
        self.remote_files_service.file_removed(mount_id, path);

        if let Some(parent_path) = path_utils::parent_path(path) {
            let _ = self.mount_path_decrypt_files(mount_id, parent_path);
        }
    }

    pub fn remote_file_copied(&self, mount_id: &str, new_path: &str, file: models::FilesFile) {
        self.remote_files_service
            .file_copied(mount_id, new_path, file);

        let new_parent_path = path_utils::parent_path(new_path);

        if let Some(new_parent_path) = new_parent_path {
            let _ = self.mount_path_decrypt_files(mount_id, new_parent_path);
        }
    }

    pub fn remote_file_moved(
        &self,
        mount_id: &str,
        path: &str,
        new_path: &str,
        file: models::FilesFile,
    ) {
        self.remote_files_service
            .file_moved(mount_id, path, new_path, file);

        let parent_path = path_utils::parent_path(path);
        let new_parent_path = path_utils::parent_path(new_path);

        if let Some(parent_path) = parent_path {
            let _ = self.mount_path_decrypt_files(mount_id, parent_path);
        }
        if parent_path != new_parent_path {
            if let Some(new_parent_path) = new_parent_path {
                let _ = self.mount_path_decrypt_files(mount_id, new_parent_path);
            }
        }
    }
}
