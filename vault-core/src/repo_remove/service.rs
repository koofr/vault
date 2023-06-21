use std::sync::Arc;

use crate::{
    repos::{errors::RemoveRepoError, ReposService},
    store,
};

use super::mutations;

pub struct RepoRemoveService {
    repos_service: Arc<ReposService>,
    store: Arc<store::Store>,
}

impl RepoRemoveService {
    pub fn new(repos_service: Arc<ReposService>, store: Arc<store::Store>) -> Self {
        Self {
            repos_service,
            store,
        }
    }

    pub fn create(&self, repo_id: &str) -> u32 {
        self.store
            .mutate(|state, notify, _, _| mutations::create(state, notify, repo_id))
    }

    pub async fn remove(&self, remove_id: u32, password: &str) -> Result<(), RemoveRepoError> {
        let repo_id = self
            .store
            .mutate(|state, notify, _, _| mutations::removing(state, notify, remove_id))?;

        let res = self.repos_service.remove_repo(&repo_id, password).await;

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store
            .mutate(|state, notify, _, _| mutations::removed(state, notify, remove_id, res))?;

        res_err
    }

    pub fn destroy(&self, remove_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::destroy(state, notify, remove_id);
        });
    }
}
