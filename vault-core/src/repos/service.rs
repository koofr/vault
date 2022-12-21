use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::cipher::Cipher;
use crate::common::state::Status;
use crate::rclone;
use crate::remote;
use crate::store;

use super::errors::BuildCipherError;
use super::errors::InvalidPasswordError;
use super::errors::RepoConfigError;
use super::errors::UnlockRepoError;
use super::errors::{RemoveRepoError, RepoLockedError, RepoNotFoundError};
use super::password_validator::check_password_validator;
use super::state::RepoConfig;
use super::{mutations, selectors};

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
        self.store.mutate(store::Event::Repos, |state| {
            state.repos.status = Status::Loading;
        });

        let repos = self.remote.get_vault_repos().await?.repos;

        self.store.mutate(store::Event::Repos, |state| {
            state.repos.status = Status::Loaded;
            mutations::repos_loaded(state, repos);
        });

        Ok(())
    }

    pub fn lock_repo(&self, repo_id: &str) -> Result<(), RepoNotFoundError> {
        self.ciphers.write().unwrap().remove(repo_id);

        self.store.mutate(store::Event::Repos, |state| {
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

    pub async fn unlock_repo(&self, repo_id: &str, password: &str) -> Result<(), UnlockRepoError> {
        let cipher = self.build_cipher(repo_id, password).await?;

        self.ciphers
            .write()
            .unwrap()
            .insert(repo_id.to_owned(), Arc::new(cipher));

        self.store.mutate(store::Event::Repos, |state| {
            mutations::unlock_repo(state, repo_id)
        })?;

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

        self.store.mutate(store::Event::Repos, |state| {
            mutations::remove_repo(state, repo_id)
        });

        Ok(())
    }

    pub async fn get_repo_config(
        &self,
        repo_id: &str,
        password: &str,
    ) -> Result<RepoConfig, RepoConfigError> {
        let _ = self.build_cipher(repo_id, password).await?;

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

    pub fn get_cipher(&self, repo_id: &str) -> Result<Arc<Cipher>, RepoLockedError> {
        let ciphers = self.ciphers.read().unwrap();
        let cipher = ciphers.get(repo_id).ok_or(RepoLockedError)?;

        Ok(cipher.clone())
    }
}
