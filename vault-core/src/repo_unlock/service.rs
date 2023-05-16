use std::sync::Arc;

use crate::{
    common::state::Status,
    repos::{
        errors::{RepoNotFoundError, UnlockRepoError},
        ReposService,
    },
    store,
};

use super::state::RepoUnlockState;

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

    pub fn init(&self, repo_id: &str) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoUnlock);

            state.repo_unlock = Some(RepoUnlockState {
                repo_id: repo_id.to_owned(),
                status: Status::Initial,
            });
        });
    }

    pub async fn unlock(&self, password: &str) -> Result<(), UnlockRepoError> {
        let repo_id = match self.store.mutate(|state, notify| {
            notify(store::Event::RepoUnlock);

            if let Some(ref mut repo_unlock) = state.repo_unlock {
                repo_unlock.status = Status::Loading;
            }

            state
                .repo_unlock
                .as_ref()
                .map(|repo_unlock| repo_unlock.repo_id.clone())
        }) {
            Some(repo_id) => repo_id,
            None => {
                return Err(UnlockRepoError::RepoNotFound(RepoNotFoundError));
            }
        };

        let res = self.repos_service.unlock_repo(&repo_id, password).await;

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoUnlock);

            if let Some(ref mut repo_unlock) = state.repo_unlock {
                repo_unlock.status = match &res {
                    Ok(()) => Status::Loaded,
                    Err(err) => Status::Error { error: err.clone() },
                };
            }
        });

        res
    }

    pub fn destroy(&self, repo_id: &str) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoUnlock);

            if state.repo_unlock.is_some() && state.repo_unlock.as_ref().unwrap().repo_id == repo_id
            {
                state.repo_unlock = None;
            }
        })
    }
}
