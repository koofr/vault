use std::collections::HashMap;

use crate::{
    remote::remote::RemoteFileTagsSetConditions,
    remote_files::state::{RemoteFile, RemoteFileType},
    repo_files::{self, state::RepoFile},
    store,
    types::{MountId, RemotePath, RepoFileId},
};

use super::{errors::SetTagsError, state::RepoFileTags};

pub const REMOTE_FILE_TAGS_KEY: &'static str = "vault";

pub fn get_remote_file_tags_set_conditions(
    remote_file: &RemoteFile,
) -> RemoteFileTagsSetConditions {
    RemoteFileTagsSetConditions {
        if_size: remote_file.size,
        if_modified: remote_file.modified,
        if_hash: remote_file.hash.clone(),
        if_old_tags: Some(HashMap::from([(
            REMOTE_FILE_TAGS_KEY.into(),
            remote_file
                .tags
                .get(REMOTE_FILE_TAGS_KEY)
                .cloned()
                .unwrap_or(vec![]),
        )])),
    }
}

pub fn select_set_tags_info(
    state: &store::State,
    file_id: &RepoFileId,
    patch: &Box<dyn Fn(&RepoFile, &mut RepoFileTags) -> Result<(), SetTagsError> + Send + Sync>,
) -> Result<
    (
        MountId,
        RemotePath,
        RepoFileTags,
        RemoteFileTagsSetConditions,
    ),
    SetTagsError,
> {
    let file =
        repo_files::selectors::select_file(state, file_id).ok_or(SetTagsError::FileNotFound)?;
    let remote_file =
        repo_files::selectors::select_remote_file(state, file).ok_or(SetTagsError::FileNotFound)?;

    let mount_id = remote_file.mount_id.clone();
    let remote_path = remote_file.path.clone();

    let mut tags = match &file.tags {
        Some(Ok(tags)) => tags.clone(),
        _ => {
            let encrypted_hash = match remote_file.typ {
                RemoteFileType::Dir => None,
                RemoteFileType::File => Some(
                    hex::decode(
                        remote_file
                            .hash
                            .as_ref()
                            .ok_or(SetTagsError::MissingEncryptedHash)?,
                    )
                    .map_err(SetTagsError::InvalidEncryptedHash)?,
                ),
            };

            RepoFileTags {
                encrypted_hash,
                hash: None,
                unknown: HashMap::new(),
            }
        }
    };

    patch(file, &mut tags)?;

    let conditions = get_remote_file_tags_set_conditions(remote_file);

    Ok((mount_id, remote_path, tags, conditions))
}
