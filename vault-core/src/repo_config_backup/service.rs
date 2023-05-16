use std::sync::Arc;

use crate::{
    common::state::Status,
    repos::{
        errors::{RepoConfigError, RepoNotFoundError},
        ReposService,
    },
    store,
};

use super::state::RepoConfigBackupState;

pub struct RepoConfigBackupService {
    repos_service: Arc<ReposService>,
    store: Arc<store::Store>,
}

impl RepoConfigBackupService {
    pub fn new(repos_service: Arc<ReposService>, store: Arc<store::Store>) -> Self {
        Self {
            repos_service,
            store,
        }
    }

    pub fn init(&self, repo_id: &str) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoConfigBackup);

            state.repo_config_backup = Some(RepoConfigBackupState {
                repo_id: repo_id.to_owned(),
                status: Status::Initial,
                config: None,
            });
        });
    }

    pub async fn generate(&self, password: &str) -> Result<(), RepoConfigError> {
        let repo_id = match self.store.mutate(|state, notify| {
            notify(store::Event::RepoConfigBackup);

            if let Some(ref mut repo_config_backup) = state.repo_config_backup {
                repo_config_backup.status = Status::Loading;
            }

            state
                .repo_config_backup
                .as_ref()
                .map(|repo_config_backup| repo_config_backup.repo_id.clone())
        }) {
            Some(repo_id) => repo_id,
            None => {
                return Err(RepoConfigError::RepoNotFound(RepoNotFoundError));
            }
        };

        let res = self.repos_service.get_repo_config(&repo_id, password).await;

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoConfigBackup);

            if let Some(ref mut repo_config_backup) = state.repo_config_backup {
                match &res {
                    Ok(config) => {
                        repo_config_backup.status = Status::Loaded;
                        repo_config_backup.config = Some(config.clone());
                    }
                    Err(err) => repo_config_backup.status = Status::Error { error: err.clone() },
                }
            }
        });

        res.map(|_| ())
    }

    pub fn destroy(&self, repo_id: &str) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoConfigBackup);

            if state.repo_config_backup.is_some()
                && state.repo_config_backup.as_ref().unwrap().repo_id == repo_id
            {
                state.repo_config_backup = None;
            }
        })
    }
}
