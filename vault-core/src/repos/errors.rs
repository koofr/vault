use thiserror::Error;

use crate::{
    remote::{self, RemoteError},
    secure_storage::errors::SecureStorageError,
    user_error::UserError,
};

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
#[error("repo unlocked")]
pub struct RepoUnlockedError;

impl UserError for RepoUnlockedError {
    fn user_error(&self) -> String {
        return "Safe Box is unlocked".into();
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
            Self::RemoteError(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LockRepoError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("already locked")]
    RepoLocked(#[from] RepoLockedError),
}

impl UserError for LockRepoError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::RepoLocked(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadReposError {
    #[error("storage error: {0}")]
    StorageError(#[from] SecureStorageError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for LoadReposError {
    fn user_error(&self) -> String {
        match self {
            Self::StorageError(err) => format!("Storage error: {}", err),
            Self::RemoteError(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum UnlockRepoError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoUnlocked(#[from] RepoUnlockedError),
    #[error("{0}")]
    InvalidPassword(#[from] InvalidPasswordError),
}

impl UserError for UnlockRepoError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::RepoUnlocked(err) => err.user_error(),
            Self::InvalidPassword(err) => err.user_error(),
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
pub enum GetCipherError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
}

impl UserError for GetCipherError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::RepoLocked(err) => err.user_error(),
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
            Self::RemoteError(err) => err.user_error(),
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
            Self::RepoNotFound(err) => err.user_error(),
            Self::InvalidPassword(err) => err.user_error(),
            Self::RemoteError(err) => err.user_error(),
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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SetAutoLockError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("storage error: {0}")]
    StorageError(#[from] SecureStorageError),
}

impl UserError for SetAutoLockError {
    fn user_error(&self) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(),
            Self::StorageError(err) => format!("Storage error: {}", err),
        }
    }
}
