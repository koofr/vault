use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{cipher::Cipher, rclone, remote, store};

use super::{
    errors::{
        BuildCipherError, InvalidPasswordError, RemoveRepoError, RepoLockedError,
        RepoNotFoundError, UnlockRepoError,
    },
    mutations,
    password_validator::check_password_validator,
    selectors,
    state::{RepoConfig, RepoUnlockMode},
};

pub struct ReposService {
    remote: Arc<remote::Remote>,
    store: Arc<store::Store>,
    ciphers: Arc<RwLock<HashMap<String, Arc<Cipher>>>>,
}

impl ReposService {
    pub fn new(remote: Arc<remote::Remote>, store: Arc<store::Store>) -> Self {
        Self {
            remote,
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
