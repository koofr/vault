use thiserror::Error;

use crate::remote;
use crate::repos::errors::RepoNotFoundError;
use crate::user_error::UserError;

#[derive(Error, Debug, Clone, UserError)]
pub enum RepoSpaceUsageError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}
