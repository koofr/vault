use std::sync::Arc;

use futures::{stream::BoxStream, StreamExt};

use crate::{remote_files::RemoteFilesService, repo_files::state::RepoFile, repos::ReposService};

use super::{
    errors::{FilesListRecursiveItemError, GetListRecursiveError},
    mutations,
    state::RepoFilesListRecursiveItem,
};

pub type RepoFilesListRecursiveItemStream = BoxStream<'static, RepoFilesListRecursiveItem>;

pub struct RepoFilesListService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
}

impl RepoFilesListService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
    ) -> Self {
        Self {
            repos_service,
            remote_files_service,
        }
    }

    pub async fn get_list_recursive(
        &self,
        file: &RepoFile,
    ) -> Result<RepoFilesListRecursiveItemStream, GetListRecursiveError> {
        let mount_id = file.mount_id.clone();
        let root_remote_path = file.remote_path.clone();
        let repo_id = file.repo_id.clone();
        let encrypted_root_path = file.encrypted_path.clone();
        let root_path = file.decrypted_path().map(ToOwned::to_owned);

        let cipher = self
            .repos_service
            .get_cipher(&file.repo_id)
            .map_err(GetListRecursiveError::RepoLocked)?;

        let remote_items_stream = self
            .remote_files_service
            .get_list_recursive(&mount_id, &root_remote_path)
            .await?;

        let repo_files_items_stream = remote_items_stream
            .map(move |item| match item {
                Ok(item) => mutations::decrypt_files_list_recursive_item(
                    &mount_id,
                    &root_remote_path,
                    &repo_id,
                    &encrypted_root_path,
                    &root_path,
                    item,
                    &cipher,
                ),
                Err(err) => RepoFilesListRecursiveItem::Error {
                    mount_id: mount_id.to_owned(),
                    remote_path: None,
                    error: FilesListRecursiveItemError::RemoteError(err),
                },
            })
            .boxed();

        Ok(repo_files_items_stream)
    }
}
