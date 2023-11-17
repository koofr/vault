use std::sync::Arc;

use crate::{
    repos::{errors::UnlockRepoError, ReposService},
    store,
    types::RepoId,
};

use super::mutations;

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

    pub fn create(&self, repo_id: RepoId) -> u32 {
        self.store
            .mutate(|state, notify, _, _| mutations::create(state, notify, repo_id))
    }

    pub async fn generate(&self, backup_id: u32, password: &str) -> Result<(), UnlockRepoError> {
        let repo_id = self
            .store
            .mutate(|state, notify, _, _| mutations::generating(state, notify, backup_id))?;

        let res = self.repos_service.get_repo_config(&repo_id, password).await;

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store
            .mutate(|state, notify, _, _| mutations::generated(state, notify, backup_id, res))?;

        res_err
    }

    pub fn destroy(&self, backup_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::destroy(state, notify, backup_id);
        });
    }
}
