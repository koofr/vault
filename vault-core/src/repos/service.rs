use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use lazy_static::lazy_static;

use crate::{
    cipher::Cipher,
    rclone,
    remote::{self, models},
    remote_files::RemoteFilesService,
    store,
    types::{DecryptedName, MountId, RemoteName, RemotePath, RepoId},
    utils::remote_path_utils,
};

use super::{
    errors::{
        BuildCipherError, CreateRepoError, InvalidPasswordError, RemoveRepoError, RepoLockedError,
        RepoNotFoundError, UnlockRepoError,
    },
    mutations,
    password_validator::{check_password_validator, generate_password_validator},
    selectors,
    state::{RepoConfig, RepoCreated, RepoUnlockMode},
};

lazy_static! {
    pub static ref DEFAULT_DIR_NAMES: Vec<DecryptedName> = vec![
        DecryptedName("My private documents".into()),
        DecryptedName("My private pictures".into()),
        DecryptedName("My private videos".into()),
    ];
}

pub struct ReposService {
    remote: Arc<remote::Remote>,
    remote_files_service: Arc<RemoteFilesService>,
    store: Arc<store::Store>,
    ciphers: Arc<RwLock<HashMap<RepoId, Arc<Cipher>>>>,
}

impl ReposService {
    pub fn new(
        remote: Arc<remote::Remote>,
        remote_files_service: Arc<RemoteFilesService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            remote,
            remote_files_service,
            store,
            ciphers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn reset(&self) {
        self.ciphers.write().unwrap().clear()
    }

    pub async fn load_repos(&self) -> Result<(), remote::RemoteError> {
        self.store.mutate(|state, notify, _, _| {
            mutations::repos_loading(state, notify);
        });

        let res = self.remote.get_vault_repos().await.map(|res| res.repos);

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::repos_loaded(state, notify, mutation_state, mutation_notify, res);
            });

        res_err
    }

    pub fn lock_repo(&self, repo_id: &RepoId) -> Result<(), RepoNotFoundError> {
        let cipher = self.ciphers.write().unwrap().remove(repo_id);

        // match needs to be separate from self.ciphers.write() otherwise
        // self.ciphers stays locked and we can deadlock
        match cipher {
            Some(cipher) => self
                .store
                .mutate(|state, notify, mutation_state, mutation_notify| {
                    mutations::lock_repo(
                        state,
                        notify,
                        mutation_state,
                        mutation_notify,
                        repo_id,
                        cipher,
                    )
                }),
            None => Err(RepoNotFoundError),
        }
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
        let cipher = self.build_cipher(repo_id, password)?;

        if matches!(mode, RepoUnlockMode::Unlock) {
            let cipher = Arc::new(cipher);

            self.ciphers
                .write()
                .unwrap()
                .insert(repo_id.to_owned(), cipher.clone());

            self.store
                .mutate(|state, notify, mutation_state, mutation_notify| {
                    mutations::unlock_repo(
                        state,
                        notify,
                        mutation_state,
                        mutation_notify,
                        repo_id,
                        cipher,
                    )
                })?;
        }

        Ok(())
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

        self.store.mutate(|state, notify, _, _| {
            mutations::repo_loaded(state, notify, repo);
        });

        let config = self.get_repo_config(&repo_id, &password).unwrap();

        Ok(RepoCreated { repo_id, config })
    }

    pub async fn remove_repo(
        &self,
        repo_id: &RepoId,
        password: &str,
    ) -> Result<(), RemoveRepoError> {
        let _ = self.build_cipher(repo_id, password)?;

        self.remote
            .remove_vault_repo(repo_id)
            .await
            .map_err(|e| match e {
                remote::RemoteError::ApiError {
                    code: remote::ApiErrorCode::NotFound,
                    ..
                } => RemoveRepoError::RepoNotFound(RepoNotFoundError),
                _ => RemoveRepoError::RemoteError(e),
            })?;

        self.ciphers.write().unwrap().remove(repo_id);

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::remove_repos(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    &[repo_id.to_owned()],
                )
            });

        Ok(())
    }

    pub fn get_repo_config(
        &self,
        repo_id: &RepoId,
        password: &str,
    ) -> Result<RepoConfig, UnlockRepoError> {
        self.unlock_repo(repo_id, password, RepoUnlockMode::Verify)?;

        self.store.with_state(|state| {
            let repo = selectors::select_repo(state, repo_id)?;

            let rclone_config = rclone::config::generate_config(&rclone::config::Config {
                name: Some(repo.name.0.clone()),
                path: repo.path.0.clone(),
                password: password.to_owned(),
                salt: repo.salt.clone(),
            });

            Ok(RepoConfig {
                name: repo.name.clone(),
                location: repo.get_location(),
                password: password.to_owned(),
                salt: repo.salt.clone(),
                rclone_config,
            })
        })
    }

    pub fn get_ciphers(&self) -> RwLockReadGuard<'_, HashMap<RepoId, Arc<Cipher>>> {
        self.ciphers.read().unwrap()
    }

    pub fn get_cipher(&self, repo_id: &RepoId) -> Result<Arc<Cipher>, RepoLockedError> {
        let ciphers = self.get_ciphers();
        let cipher = ciphers.get(repo_id).ok_or(RepoLockedError)?;

        Ok(cipher.clone())
    }
}
