use thiserror::Error;

use crate::{
    intl,
    remote::{self, RemoteError},
    secure_storage::errors::SecureStorageError,
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
#[error("repo not found")]
pub struct RepoNotFoundError;

impl UserError for RepoNotFoundError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.repos.repo_not_found.error",
            "Error shown when a Safe Box cannot be found or is no longer accessible.",
            "Safe Box not found."
        )
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("repo locked")]
pub struct RepoLockedError;

impl UserError for RepoLockedError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.repos.repo_locked.error",
            "Error shown when a Safe Box is locked and requires the Safe Key.",
            "Safe Box is locked."
        )
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("repo unlocked")]
pub struct RepoUnlockedError;

impl UserError for RepoUnlockedError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.repos.repo_unlocked.error",
            "Error shown when attempting to unlock a Safe Box that is already unlocked.",
            "Safe Box is unlocked."
        )
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("invalid password")]
pub struct InvalidPasswordError;

impl UserError for InvalidPasswordError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.repos.invalid_password.error",
            "Error shown when the provided Safe Key is incorrect.",
            "Safe Key is not correct."
        )
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(_) => intl::format_message!(
                intl_service,
                "core.repos.repo_not_found.error",
                "Error shown when a Safe Box cannot be found or is no longer accessible.",
                "Safe Box not found."
            ),
            Self::RemoteError(err) => err.user_error(intl_service),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::StorageError(err) => err.user_error(intl_service),
            Self::RemoteError(err) => err.user_error(intl_service),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoUnlocked(err) => err.user_error(intl_service),
            Self::InvalidPassword(err) => err.user_error(intl_service),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CreateRepoError {
    #[error("{0}")]
    RemoteError(#[from] remote::RemoteError),
}

impl UserError for CreateRepoError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RemoteError(remote::RemoteError::ApiError {
                code: remote::ApiErrorCode::VaultReposAlreadyExists,
                ..
            }) => intl::format_message!(
                intl_service,
                "core.repos.create_repo_already_exists.error",
                "Error shown when creating a Safe Box at a location that already contains one.",
                "This location is already a Safe Box."
            ),
            Self::RemoteError(remote::RemoteError::ApiError {
                code: remote::ApiErrorCode::VaultReposMaxTotalLimitExceeded,
                ..
            }) => intl::format_message!(
                intl_service,
                "core.repos.create_repo_max_total_limit_exceeded.error",
                "Error shown when the Safe Box creation limit for the account is exceeded.",
                "You cannot create more Safe Boxes. Please upgrade your account."
            ),
            Self::RemoteError(err) => err.user_error(intl_service),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::InvalidPassword(err) => err.user_error(intl_service),
            Self::RemoteError(err) => err.user_error(intl_service),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::StorageError(err) => err.user_error(intl_service),
        }
    }
}
