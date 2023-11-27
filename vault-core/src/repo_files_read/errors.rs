use thiserror::Error;

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    remote::RemoteError,
    repos::errors::{RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
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

impl UserError for GetFilesReaderError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::RepoLocked(err) => err.user_error(),
            Self::FileNotFound => self.to_string(),
            Self::FilesEmpty => self.to_string(),
            Self::DecryptFilenameError(err) => err.user_error(),
            Self::DecryptSizeError(err) => err.user_error(),
            Self::RemoteError(err) => err.user_error(),
            Self::IOError(err) => err.to_string(),
            Self::Aborted => self.to_string(),
        }
    }
}

impl From<&std::io::Error> for GetFilesReaderError {
    fn from(err: &std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::Interrupted => GetFilesReaderError::Aborted,
            _ => GetFilesReaderError::IOError(err.to_string()),
        }
    }
}
