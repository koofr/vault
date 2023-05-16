use thiserror::Error;

use crate::{remote, repos::errors::RepoNotFoundError, user_error::UserError};

#[derive(Error, Debug, Clone, UserError)]
pub enum RepoSpaceUsageError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}
