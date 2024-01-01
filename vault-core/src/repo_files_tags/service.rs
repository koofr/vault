use std::{collections::HashMap, sync::Arc};

use crate::{
    remote::ApiErrorCode,
    remote_files::RemoteFilesService,
    repo_files::{self, state::RepoFile},
    repo_files_tags::selectors,
    repos::ReposService,
    store,
    types::{EncryptedPath, RepoId},
};

use super::{errors::SetTagsError, state::RepoFileTags};

pub struct RepoFilesTagsService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    store: Arc<store::Store>,
}

impl RepoFilesTagsService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            repos_service,
            remote_files_service,
            store,
        }
    }

    pub async fn set_tags(
        &self,
        repo_id: &RepoId,
        path: &EncryptedPath,
        patch: Box<dyn Fn(&RepoFile, &mut RepoFileTags) -> Result<(), SetTagsError> + Send + Sync>,
    ) -> Result<(), SetTagsError> {
        let cipher = self.repos_service.get_cipher(&repo_id)?;

        let file_id = repo_files::selectors::get_file_id(&repo_id, &path);

        let max_retries = self
            .store
            .with_state(|state| state.config.repo_files_tags.set_tags_max_retries);

        let mut retry_count = 0;

        loop {
            let (mount_id, remote_path, tags, conditions) = self
                .store
                .with_state(|state| selectors::select_set_tags_info(state, &file_id, &patch))?;

            let encrypted_tags = tags.to_string(&cipher)?;

            let remote_tags = HashMap::from([(
                selectors::REMOTE_FILE_TAGS_KEY.to_owned(),
                vec![encrypted_tags],
            )]);

            let res = self
                .remote_files_service
                .set_tags(&mount_id, &remote_path, remote_tags, conditions)
                .await;

            match res {
                Ok(()) => return Ok(()),
                Err(err) => {
                    retry_count += 1;

                    if err.is_api_error_code(ApiErrorCode::Conflict) && retry_count < max_retries {
                        self.remote_files_service
                            .load_files(&mount_id, &remote_path)
                            .await
                            .map_err(SetTagsError::RemoteError)?;
                    } else {
                        return Err(SetTagsError::RemoteError(err));
                    }
                }
            }
        }
    }

    pub async fn set_tags_hash(
        &self,
        repo_id: &RepoId,
        path: &EncryptedPath,
        remote_file_hash: String,
        hash: Vec<u8>,
    ) -> Result<(), SetTagsError> {
        let remote_file_hash = Arc::new(remote_file_hash.to_owned());
        let hash = Arc::new(hash);

        self.set_tags(
            repo_id,
            path,
            Box::new(move |file, tags| {
                if file.remote_hash.as_deref() != Some(&remote_file_hash) {
                    return Err(SetTagsError::EncryptedHashMismatch {
                        expected_encrypted_hash: (*remote_file_hash).clone(),
                        encrypted_hash: file.remote_hash.clone(),
                    });
                }

                tags.hash = Some((*hash).clone());

                Ok(())
            }),
        )
        .await
    }
}
