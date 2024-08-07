use thiserror::Error;

use crate::{remote, repos::errors::RepoNotFoundError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RepoSpaceUsageError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}

impl UserError for RepoSpaceUsageError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::RemoteError(err) => err.user_error(),
        }
    }
}
