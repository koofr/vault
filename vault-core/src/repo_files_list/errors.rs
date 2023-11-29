use thiserror::Error;

use crate::{
    cipher::errors::DecryptFilenameError,
    remote::RemoteError,
    repos::errors::{GetCipherError, RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum FilesListRecursiveItemError {
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for FilesListRecursiveItemError {
    fn user_error(&self) -> String {
        match self {
            Self::DecryptFilenameError(err) => err.user_error(),
            Self::RemoteError(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GetListRecursiveError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for GetListRecursiveError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::RepoLocked(err) => err.user_error(),
            Self::DecryptFilenameError(err) => err.user_error(),
            Self::RemoteError(err) => err.user_error(),
        }
    }
}

impl From<GetCipherError> for GetListRecursiveError {
    fn from(err: GetCipherError) -> Self {
        match err {
            GetCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            GetCipherError::RepoLocked(err) => Self::RepoLocked(err),
        }
    }
}
