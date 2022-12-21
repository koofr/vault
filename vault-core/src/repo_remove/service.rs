use std::sync::Arc;

use crate::{
    common::state::Status,
    repos::{
        errors::{RemoveRepoError, RepoNotFoundError},
        ReposService,
    },
    store,
};

use super::state::RepoRemoveState;

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

    pub fn init(&self, repo_id: &str) {
        self.store.mutate(store::Event::RepoRemove, |state| {
            state.repo_remove = Some(RepoRemoveState {
                repo_id: repo_id.to_owned(),
                status: Status::Initial,
            });
        });
    }

    pub async fn remove(&self, password: &str) -> Result<(), RemoveRepoError> {
        let repo_id = match self.store.mutate(store::Event::RepoRemove, |state| {
            if let Some(ref mut repo_remove) = state.repo_remove {
                repo_remove.status = Status::Loading;
            }

            state
                .repo_remove
                .as_ref()
                .map(|repo_remove| repo_remove.repo_id.clone())
        }) {
            Some(repo_id) => repo_id,
            None => {
                return Err(RemoveRepoError::RepoNotFound(RepoNotFoundError));
            }
        };

        let res = self.repos_service.remove_repo(&repo_id, password).await;

        self.store.mutate(store::Event::RepoRemove, |state| {
            if let Some(ref mut repo_remove) = state.repo_remove {
                repo_remove.status = match &res {
                    Ok(()) => Status::Loaded,
                    Err(err) => Status::Error { error: err.clone() },
                };
            }
        });

        res
    }

    pub fn destroy(&self, repo_id: &str) {
        self.store.mutate(store::Event::RepoRemove, |state| {
            if state.repo_remove.is_some() && state.repo_remove.as_ref().unwrap().repo_id == repo_id
            {
                state.repo_remove = None;
            }
        })
    }
}
