use thiserror::Error;

use crate::{remote, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
#[error("repo not found")]
pub struct RepoNotFoundError;

impl UserError for RepoNotFoundError {
    fn user_error(&self) -> String {
        return "Safe Box not found".into();
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("repo locked")]
pub struct RepoLockedError;

impl UserError for RepoLockedError {
    fn user_error(&self) -> String {
        return "Safe Box is locked".into();
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("invalid password")]
pub struct InvalidPasswordError;

impl UserError for InvalidPasswordError {
    fn user_error(&self) -> String {
        String::from("Safe Key is not correct.")
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BuildCipherError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    InvalidPassword(#[from] InvalidPasswordError),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RepoInfoError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}

impl UserError for RepoInfoError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(_) => String::from("Safe Box not found."),
            _ => self.to_string(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum UnlockRepoError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    InvalidPassword(#[from] InvalidPasswordError),
}

impl UserError for UnlockRepoError {
    fn user_error(&self) -> String {
        match self {
            Self::InvalidPassword(err) => err.user_error(),
            _ => self.to_string(),
        }
    }
}

impl From<BuildCipherError> for UnlockRepoError {
    fn from(err: BuildCipherError) -> Self {
        match err {
            BuildCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            BuildCipherError::InvalidPassword(err) => Self::InvalidPassword(err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CreateRepoError {
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}

impl UserError for CreateRepoError {
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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RemoveRepoError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    InvalidPassword(#[from] InvalidPasswordError),
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}

impl UserError for RemoveRepoError {
    fn user_error(&self) -> String {
        match self {
            Self::InvalidPassword(err) => err.user_error(),
            _ => self.to_string(),
        }
    }
}

impl From<BuildCipherError> for RemoveRepoError {
    fn from(err: BuildCipherError) -> Self {
        match err {
            BuildCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            BuildCipherError::InvalidPassword(err) => Self::InvalidPassword(err),
        }
    }
}
