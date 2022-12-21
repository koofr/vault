use thiserror::Error;

use crate::remote;
use crate::user_error::UserError;

#[derive(Error, Debug, Clone)]
pub enum RepoCreateError {
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}

impl UserError for RepoCreateError {
    fn user_error(&self) -> String {
        match self {
            Self::RemoteError(remote::RemoteError::ApiError {
                code: remote::ApiErrorCode::VaultReposAlreadyExists,
                ..
            }) => String::from("This location is already a Safe Box."),
            Self::RemoteError(remote::RemoteError::ApiError {
                code: remote::ApiErrorCode::VaultReposMaxTotalLimitExceeded,
                ..
            }) => String::from("You cannot create more Safe Boxes. Please upgrade your account."),
            _ => self.to_string(),
        }
    }
}
