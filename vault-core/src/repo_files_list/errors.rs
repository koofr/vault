use thiserror::Error;

use crate::{
    cipher::errors::DecryptFilenameError,
    remote::RemoteError,
    repos::errors::{RepoLockedError, RepoNotFoundError},
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
            _ => self.to_string(),
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
            Self::DecryptFilenameError(err) => err.user_error(),
            _ => self.to_string(),
        }
    }
}
