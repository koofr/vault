use std::sync::Arc;

use futures::{future, StreamExt};

use crate::{
    remote::{models::FilesListRecursiveItem, RemoteError},
    remote_files::RemoteFilesService,
    store,
    types::RepoId,
};

use super::{errors::RepoSpaceUsageError, mutations};

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

    pub fn create(&self, repo_id: RepoId) -> u32 {
        self.store
            .mutate(|state, notify, _, _| mutations::create(state, notify, repo_id))
    }

    pub async fn calculate(&self, usage_id: u32) -> Result<(), RepoSpaceUsageError> {
        let repo_location = self
            .store
            .mutate(|state, notify, _, _| mutations::calculating(state, notify, usage_id))?;

        let items_stream = match self
            .remote_files_service
            .get_list_recursive(&repo_location.mount_id, &repo_location.path)
            .await
            .map_err(RepoSpaceUsageError::RemoteError)
        {
            Ok(items_stream) => items_stream,
            Err(err) => {
                self.store.mutate(|state, notify, _, _| {
                    mutations::calculated(state, notify, usage_id, None, Err(err.clone()))
                })?;

                return Err(err);
            }
        };

        let mut space_used = 0;
        let mut res: Result<(), RepoSpaceUsageError> = Ok(());

        items_stream
            .for_each(|item| {
                match item {
                    Ok(item) => match item {
                        FilesListRecursiveItem::File { file, .. } => {
                            space_used += file.size;
                        }
                        FilesListRecursiveItem::Error { error, .. } => {
                            res = Err(RepoSpaceUsageError::RemoteError(
                                RemoteError::from_api_error_details(error, None, None),
                            ));
                        }
                    },
                    Err(err) => {
                        res = Err(RepoSpaceUsageError::RemoteError(err));
                    }
                };

                future::ready(())
            })
            .await;

        self.store.mutate(|state, notify, _, _| {
            mutations::calculated(state, notify, usage_id, Some(space_used), res.clone())
        })?;

        res
    }

    pub fn destroy(&self, usage_id: u32) {
        self.store
            .mutate(|state, notify, _, _| mutations::destroy(state, notify, usage_id));
    }
}
