use thiserror::Error;

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    remote::RemoteError,
    repos::errors::{RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone, UserError)]
pub enum GetFilesReaderError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("file not found")]
    FileNotFound,
    #[error("files empty")]
    FilesEmpty,
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    DecryptSizeError(#[from] DecryptSizeError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
    #[error("{0}")]
    IOError(String),
    #[error("aborted")]
    Aborted,
}

impl From<&std::io::Error> for GetFilesReaderError {
    fn from(err: &std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::Interrupted => GetFilesReaderError::Aborted,
            _ => GetFilesReaderError::IOError(err.to_string()),
        }
    }
}
