use std::sync::Arc;

use crate::{
    repos::{errors::UnlockRepoError, ReposService},
    store,
    types::RepoId,
};

use super::{mutations, state::RepoUnlockOptions};

pub struct RepoUnlockService {
    repos_service: Arc<ReposService>,
    store: Arc<store::Store>,
}

impl RepoUnlockService {
    pub fn new(repos_service: Arc<ReposService>, store: Arc<store::Store>) -> Self {
        Self {
            repos_service,
            store,
        }
    }

    pub fn create(&self, repo_id: RepoId, options: RepoUnlockOptions) -> u32 {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoUnlock);

            mutations::create(state, notify, repo_id, options)
        })
    }

    pub async fn unlock(&self, unlock_id: u32, password: &str) -> Result<(), UnlockRepoError> {
        let (repo_id, mode) = self
            .store
            .mutate(|state, notify, _, _| mutations::unlocking(state, notify, unlock_id))?;

        let res = self
            .repos_service
            .unlock_repo(&repo_id, password, mode)
            .await;

        self.store.mutate(|state, notify, _, _| {
            mutations::unlocked(state, notify, unlock_id, res.clone());
        });

        res
    }

    pub fn destroy(&self, unlock_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::destroy(state, notify, unlock_id);
        })
    }
}
