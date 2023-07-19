use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{
    cipher::Cipher,
    rclone,
    remote::{self, models},
    remote_files::RemoteFilesService,
    store,
    utils::path_utils,
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

const DEFAULT_DIR_NAMES: &'static [&'static str] = &[
    "My private documents",
    "My private pictures",
    "My private videos",
];

pub struct ReposService {
    remote: Arc<remote::Remote>,
    remote_files_service: Arc<RemoteFilesService>,
    store: Arc<store::Store>,
    ciphers: Arc<RwLock<HashMap<String, Arc<Cipher>>>>,
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
            notify(store::Event::Repos);

            mutations::repos_loading(state);
        });

        let repos = self.remote.get_vault_repos().await?.repos;

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Repos);

            mutations::repos_loaded(state, repos);
        });

        Ok(())
    }

    pub fn lock_repo(&self, repo_id: &str) -> Result<(), RepoNotFoundError> {
        self.ciphers.write().unwrap().remove(repo_id);

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Repos);

            mutations::lock_repo(state, repo_id)
        })
    }

    pub async fn build_cipher(
        &self,
        repo_id: &str,
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

        let cipher = Cipher::new(password, salt.as_deref());

        if !check_password_validator(&cipher, &password_validator, &password_validator_encrypted)
            .await
        {
            return Err(BuildCipherError::InvalidPassword(InvalidPasswordError));
        }

        Ok(cipher)
    }

    pub async fn unlock_repo(
        &self,
        repo_id: &str,
        password: &str,
        mode: RepoUnlockMode,
    ) -> Result<(), UnlockRepoError> {
        let cipher = self.build_cipher(repo_id, password).await?;

        if matches!(mode, RepoUnlockMode::Unlock) {
            self.ciphers
                .write()
                .unwrap()
                .insert(repo_id.to_owned(), Arc::new(cipher));

            self.store.mutate(|state, notify, _, _| {
                notify(store::Event::Repos);

                mutations::unlock_repo(state, repo_id)
            })?;
        }

        Ok(())
    }

    pub async fn create_repo(
        &self,
        mount_id: &str,
        path: &str,
        password: &str,
        salt: Option<&str>,
    ) -> Result<RepoCreated, CreateRepoError> {
        let already_exists = match (
            path_utils::parent_path(&path),
            path_utils::path_to_name(&path),
        ) {
            (Some(parent_path), Some(name)) => {
                match self.remote.create_dir(&mount_id, parent_path, name).await {
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

        let cipher = Cipher::new(&password, salt.as_deref());

        let (password_validator, password_validator_encrypted) =
            generate_password_validator(&cipher).await;

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
            for name in DEFAULT_DIR_NAMES {
                let encrypted_name = cipher.encrypt_filename(name);

                self.remote_files_service
                    .create_dir_name(&mount_id, &path, &encrypted_name)
                    .await?;
            }
        }

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Repos);

            mutations::repo_loaded(state, repo);
        });

        let config = self.get_repo_config(&repo_id, &password).await.unwrap();

        Ok(RepoCreated { repo_id, config })
    }

    pub async fn remove_repo(&self, repo_id: &str, password: &str) -> Result<(), RemoveRepoError> {
        let _ = self.build_cipher(repo_id, password).await?;

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

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Repos);

            mutations::remove_repo(state, repo_id)
        });

        Ok(())
    }

    pub async fn get_repo_config(
        &self,
        repo_id: &str,
        password: &str,
    ) -> Result<RepoConfig, UnlockRepoError> {
        self.unlock_repo(repo_id, password, RepoUnlockMode::Verify)
            .await?;

        self.store.with_state(|state| {
            let repo = selectors::select_repo(state, repo_id)?;

            let rclone_config = rclone::config::generate_config(&rclone::config::Config {
                name: Some(repo.name.clone()),
                path: repo.path.clone(),
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

    pub fn get_ciphers(&self) -> RwLockReadGuard<'_, HashMap<String, Arc<Cipher>>> {
        self.ciphers.read().unwrap()
    }

    pub fn get_cipher(&self, repo_id: &str) -> Result<Arc<Cipher>, RepoLockedError> {
        let ciphers = self.get_ciphers();
        let cipher = ciphers.get(repo_id).ok_or(RepoLockedError)?;

        Ok(cipher.clone())
    }
}
