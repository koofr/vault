use crate::{
    cipher::errors::DecryptFilenameError,
    repo_files::state::RepoFile,
    types::{DecryptedPath, MountId, RemotePath},
};

use super::errors::FilesListRecursiveItemError;

#[derive(Clone, Debug, PartialEq)]
pub enum RepoFilesListRecursiveItem {
    File {
        relative_repo_path: Result<DecryptedPath, DecryptFilenameError>,
        file: RepoFile,
    },
    Error {
        mount_id: MountId,
        remote_path: Option<RemotePath>,
        error: FilesListRecursiveItemError,
    },
}
