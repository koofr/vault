use thiserror::Error;

use crate::remote;
use crate::user_error::UserError;

#[derive(Error, Debug, Clone, PartialEq, UserError)]
#[error("repo not found")]
pub struct RepoNotFoundError;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("repo locked")]
pub struct RepoLockedError;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("invalid password")]
pub struct InvalidPasswordError;

impl UserError for InvalidPasswordError {
    fn user_error(&self) -> String {
        String::from("Safe Key is not correct.")
    }
}

#[derive(Error, Debug, Clone)]
pub enum BuildCipherError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    InvalidPassword(#[from] InvalidPasswordError),
}

#[derive(Error, Debug, Clone)]
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

#[derive(Error, Debug, Clone)]
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

#[derive(Error, Debug, Clone)]
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

#[derive(Error, Debug, Clone)]
pub enum RepoConfigError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    InvalidPassword(#[from] InvalidPasswordError),
}

impl UserError for RepoConfigError {
    fn user_error(&self) -> String {
        match self {
            Self::InvalidPassword(err) => err.user_error(),
            _ => self.to_string(),
        }
    }
}

impl From<BuildCipherError> for RepoConfigError {
    fn from(err: BuildCipherError) -> Self {
        match err {
            BuildCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            BuildCipherError::InvalidPassword(err) => Self::InvalidPassword(err),
        }
    }
}
