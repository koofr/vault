use std::sync::Arc;

use futures::{future, StreamExt};

use crate::{
    common::state::Status,
    remote::{models::FilesListRecursiveItem, RemoteError},
    remote_files::RemoteFilesService,
    repos::{errors::RepoNotFoundError, selectors as repos_selectors},
    store,
};

use super::{errors::RepoSpaceUsageError, state::RepoSpaceUsageState};

pub struct RepoSpaceUsageService {
    remote_files_service: Arc<RemoteFilesService>,
    store: Arc<store::Store>,
}

impl RepoSpaceUsageService {
    pub fn new(remote_files_service: Arc<RemoteFilesService>, store: Arc<store::Store>) -> Self {
        Self {
            remote_files_service,
            store,
        }
    }

    pub fn init(&self, repo_id: &str) {
        self.store.mutate(store::Event::RepoSpaceUsage, |state| {
            state.repo_space_usage = Some(RepoSpaceUsageState {
                repo_id: repo_id.to_owned(),
                status: Status::Initial,
                space_used: None,
            });
        });
    }

    pub async fn calculate(&self) -> Result<(), RepoSpaceUsageError> {
        let repo_location = match self.store.mutate(store::Event::RepoSpaceUsage, |state| {
            if let Some(ref mut repo_space_usage) = state.repo_space_usage {
                repo_space_usage.status = Status::Loading;
            }

            state
                .repo_space_usage
                .as_ref()
                .map(|repo_space_usage| &repo_space_usage.repo_id)
                .and_then(|repo_id| repos_selectors::select_repo(state, repo_id).ok())
                .map(|repo| repo.get_location())
        }) {
            Some(repo_location) => repo_location,
            None => {
                return Err(RepoSpaceUsageError::RepoNotFound(RepoNotFoundError));
            }
        };

        let items_stream = match self
            .remote_files_service
            .get_list_recursive(&repo_location.mount_id, &repo_location.path)
            .await
            .map_err(RepoSpaceUsageError::RemoteError)
        {
            Ok(items_stream) => items_stream,
            Err(err) => {
                self.store.mutate(store::Event::RepoSpaceUsage, |state| {
                    if let Some(ref mut repo_space_usage) = state.repo_space_usage {
                        repo_space_usage.status = Status::Error { error: err.clone() };
                    }
                });

                return Err(err);
            }
        };

        let mut space_used = 0;
        let mut last_error: Option<RepoSpaceUsageError> = None;

        items_stream
            .for_each(|item| {
                match item {
                    Ok(item) => match item {
                        FilesListRecursiveItem::File { file, .. } => {
                            space_used += file.size;
                        }
                        FilesListRecursiveItem::Error { error, .. } => {
                            last_error = Some(RepoSpaceUsageError::RemoteError(
                                RemoteError::from_api_error_details(error, None),
                            ));
                        }
                    },
                    Err(err) => {
                        last_error = Some(RepoSpaceUsageError::RemoteError(err));
                    }
                };

                future::ready(())
            })
            .await;

        self.store.mutate(store::Event::RepoSpaceUsage, |state| {
            if let Some(ref mut repo_space_usage) = state.repo_space_usage {
                repo_space_usage.status = match &last_error {
                    Some(err) => Status::Error { error: err.clone() },
                    None => Status::Loaded,
                };
                repo_space_usage.space_used = Some(space_used);
            }
        });

        match last_error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    pub fn destroy(&self, repo_id: &str) {
        self.store.mutate(store::Event::RepoSpaceUsage, |state| {
            if state.repo_space_usage.is_some()
                && state.repo_space_usage.as_ref().unwrap().repo_id == repo_id
            {
                state.repo_space_usage = None;
            }
        })
    }
}
