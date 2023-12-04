use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;

use crate::{
    cipher::Cipher,
    rclone,
    remote::{self, models},
    remote_files::RemoteFilesService,
    runtime,
    secure_storage::{errors::SecureStorageError, SecureStorageService},
    store,
    types::{DecryptedName, MountId, RemoteName, RemotePath, RepoId},
    utils::remote_path_utils,
};

use super::{
    errors::{
        BuildCipherError, CreateRepoError, GetCipherError, InvalidPasswordError, LoadReposError,
        LockRepoError, RemoveRepoError, RepoNotFoundError, SetAutoLockError, UnlockRepoError,
    },
    mutations,
    password_validator::{check_password_validator, generate_password_validator},
    selectors,
    state::{Repo, RepoAutoLock, RepoConfig, RepoCreated, RepoUnlockMode},
};

lazy_static! {
    pub static ref DEFAULT_DIR_NAMES: Vec<DecryptedName> = vec![
        DecryptedName("My private documents".into()),
        DecryptedName("My private pictures".into()),
        DecryptedName("My private videos".into()),
    ];
}

type RepoAutoLocks = HashMap<RepoId, RepoAutoLock>;

pub const REPO_AUTO_LOCKS_STORAGE_KEY: &str = "vaultRepoAutoLocks";

pub struct ReposService {
    remote: Arc<remote::Remote>,
    remote_files_service: Arc<RemoteFilesService>,
    secure_storage_service: Arc<SecureStorageService>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,
}

impl ReposService {
    pub fn new(
        remote: Arc<remote::Remote>,
        remote_files_service: Arc<RemoteFilesService>,
        secure_storage_service: Arc<SecureStorageService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Self {
        Self {
            remote,
            remote_files_service,
            secure_storage_service,
            store,
            runtime,
        }
    }

    pub fn get_auto_locks(&self) -> Result<RepoAutoLocks, SecureStorageError> {
        self.secure_storage_service
            .get::<RepoAutoLocks>(REPO_AUTO_LOCKS_STORAGE_KEY)
            .map(|x| x.unwrap_or_default())
    }

    pub async fn load_repos(&self) -> Result<(), LoadReposError> {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::repos_loading(state, notify, mutation_state, mutation_notify);
            });

        let res = self.remote.get_vault_repos().await.map(|res| res.repos);

        let res_err = res
            .as_ref()
            .map(|_| ())
            .map_err(|err| LoadReposError::from(err.to_owned()));

        let auto_locks = self.get_auto_locks()?;

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::repos_loaded(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    res,
                    &auto_locks,
                );
            });

        res_err
    }

    pub fn lock_repo(&self, repo_id: &RepoId) -> Result<(), LockRepoError> {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::lock_repo(state, notify, mutation_state, mutation_notify, repo_id)
            })
    }

    pub fn build_cipher(
        &self,
        repo_id: &RepoId,
        password: &str,
    ) -> Result<Cipher, BuildCipherError> {
        let (salt, password_validator, password_validator_encrypted) =
            self.store.with_state(|state| {
                selectors::select_repo(state, repo_id).map(|repo| {
                    (
                        repo.salt.clone(),
                        repo.password_validator.clone(),
                        repo.password_validator_encrypted.clone(),
                    )
                })
            })?;

        let cipher = Cipher::new(vault_crypto::Cipher::new(password, salt.as_deref()));

        if !check_password_validator(&cipher, &password_validator, &password_validator_encrypted) {
            return Err(BuildCipherError::InvalidPassword(InvalidPasswordError));
        }

        Ok(cipher)
    }

    pub fn unlock_repo(
        &self,
        repo_id: &RepoId,
        password: &str,
        mode: RepoUnlockMode,
    ) -> Result<(), UnlockRepoError> {
        match mode {
            RepoUnlockMode::Unlock => {
                self.store.mutate(|state, _, _, _| {
                    mutations::check_unlock_repo(state, repo_id).map(|_| ())
                })?;

                let cipher = Arc::new(self.build_cipher(repo_id, password)?);

                let now = self.runtime.now();

                self.store
                    .mutate(|state, notify, mutation_state, mutation_notify| {
                        mutations::unlock_repo(
                            state,
                            notify,
                            mutation_state,
                            mutation_notify,
                            repo_id,
                            cipher,
                            now,
                        )
                    })?;

                Ok(())
            }
            RepoUnlockMode::Verify => {
                self.build_cipher(repo_id, password)?;

                Ok(())
            }
        }
    }

    pub async fn create_repo(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        password: &str,
        salt: Option<&str>,
    ) -> Result<RepoCreated, CreateRepoError> {
        let already_exists = match (
            remote_path_utils::parent_path(&path),
            remote_path_utils::path_to_name(&path),
        ) {
            (Some(parent_path), Some(name)) => {
                match self.remote.create_dir(&mount_id, &parent_path, name).await {
                    Ok(_) => false,
                    Err(remote::RemoteError::ApiError {
                        code: remote::ApiErrorCode::AlreadyExists,
                        ..
                    }) => true,
                    Err(err) => {
                        return Err(CreateRepoError::RemoteError(err));
                    }
                }
            }
            _ => false,
        };

        let cipher = Cipher::new(vault_crypto::Cipher::new(password, salt.as_deref()));

        let (password_validator, password_validator_encrypted) =
            generate_password_validator(&cipher);

        let repo = self
            .remote
            .create_vault_repo(models::VaultRepoCreate {
                mount_id: mount_id.to_owned(),
                path: path.to_owned(),
                salt: salt.map(str::to_string),
                password_validator,
                password_validator_encrypted,
            })
            .await?;
        let repo_id = repo.id.clone();

        if !already_exists {
            for name in DEFAULT_DIR_NAMES.iter() {
                let encrypted_name = cipher.encrypt_filename(name);

                self.remote_files_service
                    .create_dir_name(&mount_id, &path, RemoteName(encrypted_name.0))
                    .await?;
            }
        }

        let config = self
            .store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::repo_created(state, notify, mutation_state, mutation_notify, repo);

                let repo = selectors::select_repo(state, &repo_id).unwrap();

                self.generate_repo_config(repo, &password)
            });

        Ok(RepoCreated { repo_id, config })
    }

    pub async fn remove_repo(
        &self,
        repo_id: &RepoId,
        password: &str,
    ) -> Result<(), RemoveRepoError> {
        let _ = self.build_cipher(repo_id, password)?;

        let res = self
            .remote
            .remove_vault_repo(repo_id)
            .await
            .map_err(|e| match e {
                remote::RemoteError::ApiError {
                    code: remote::ApiErrorCode::NotFound,
                    ..
                } => RemoveRepoError::RepoNotFound(RepoNotFoundError),
                _ => RemoveRepoError::RemoteError(e),
            });

        match res {
            Ok(()) | Err(RemoveRepoError::RepoNotFound(..)) => {
                self.store
                    .mutate(|state, notify, mutation_state, mutation_notify| {
                        mutations::repo_removed(
                            state,
                            notify,
                            mutation_state,
                            mutation_notify,
                            repo_id.to_owned(),
                        )
                    })?
            }
            _ => {}
        }

        Ok(())
    }

    pub fn touch_repo(&self, repo_id: &RepoId) -> Result<(), RepoNotFoundError> {
        let now = self.runtime.now();

        self.store
            .mutate(|state, _, _, _| mutations::touch_repo(state, repo_id, now))
    }

    pub fn set_auto_lock(
        &self,
        repo_id: &RepoId,
        auto_lock: RepoAutoLock,
    ) -> Result<(), SetAutoLockError> {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_auto_lock(state, notify, repo_id, auto_lock)
                .map_err(SetAutoLockError::RepoNotFound)?;

            let auto_locks = selectors::select_auto_locks(state);

            self.secure_storage_service
                .set(REPO_AUTO_LOCKS_STORAGE_KEY, &auto_locks)?;

            Ok(())
        })
    }

    pub fn set_default_auto_lock(&self, auto_lock: RepoAutoLock) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_default_auto_lock(state, notify, auto_lock);
        });
    }

    pub fn get_repo_config(
        &self,
        repo_id: &RepoId,
        password: &str,
    ) -> Result<RepoConfig, UnlockRepoError> {
        self.unlock_repo(repo_id, password, RepoUnlockMode::Verify)?;

        self.store.with_state(|state| {
            let repo = selectors::select_repo(state, repo_id)?;

            Ok(self.generate_repo_config(&repo, password))
        })
    }

    fn generate_repo_config(&self, repo: &Repo, password: &str) -> RepoConfig {
        let rclone_config = rclone::config::generate_config(&rclone::config::Config {
            name: Some(repo.name.0.clone()),
            path: repo.path.0.clone(),
            password: password.to_owned(),
            salt: repo.salt.clone(),
        });

        RepoConfig {
            name: repo.name.clone(),
            location: repo.get_location(),
            password: password.to_owned(),
            salt: repo.salt.clone(),
            rclone_config,
        }
    }

    pub fn get_cipher(&self, repo_id: &RepoId) -> Result<Arc<Cipher>, GetCipherError> {
        self.store
            .with_state(|state| selectors::select_cipher_owned(state, repo_id))
    }
}
