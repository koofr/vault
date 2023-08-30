use crate::{cipher::errors::DecryptFilenameError, repo_files::state::RepoFile};

use super::errors::FilesListRecursiveItemError;

#[derive(Clone, Debug, PartialEq)]
pub enum RepoFilesListRecursiveItem {
    File {
        relative_repo_path: Result<String, DecryptFilenameError>,
        file: RepoFile,
    },
    Error {
        mount_id: String,
        remote_path: Option<String>,
        error: FilesListRecursiveItemError,
    },
}
