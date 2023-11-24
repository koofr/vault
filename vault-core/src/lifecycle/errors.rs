use thiserror::Error;

use crate::{
    oauth2::errors::OAuth2Error, remote::RemoteError, secure_storage::errors::SecureStorageError,
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadError {
    #[error("{0}")]
    OAuth2LoadError(OAuth2Error),
    #[error("{0}")]
    OnLoginError(OnLoginError),
}

impl UserError for LoadError {
    fn user_error(&self) -> String {
        match self {
            Self::OAuth2LoadError(err) => err.user_error(),
            Self::OnLoginError(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OnLoginError {
    #[error("{0}")]
    LoadUserError(RemoteError),
    #[error("{0}")]
    LoadReposError(RemoteError),
    #[error("{0}")]
    LoadSpaceUsageError(RemoteError),
}

impl UserError for OnLoginError {
    fn user_error(&self) -> String {
        match self {
            Self::LoadUserError(err) => format!("Failed to load user: {}", err.user_error()),
            Self::LoadReposError(err) => format!("Failed to load safe boxes: {}", err.user_error()),
            Self::LoadSpaceUsageError(err) => {
                format!("Failed to load space usage: {}", err.user_error())
            }
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LogoutError {
    #[error("{0}")]
    OAuth2LogoutError(OAuth2Error),
    #[error("{0}")]
    OnLogoutError(#[from] OnLogoutError),
}

impl UserError for LogoutError {
    fn user_error(&self) -> String {
        match self {
            Self::OAuth2LogoutError(err) => err.user_error(),
            Self::OnLogoutError(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OnLogoutError {
    #[error("{0}")]
    ClearStorageError(SecureStorageError),
}

impl UserError for OnLogoutError {
    fn user_error(&self) -> String {
        match self {
            Self::ClearStorageError(err) => format!("Failed to clear storage: {}", err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OAuth2FinishFlowUrlError {
    #[error("{0}")]
    OAuth2Error(#[from] OAuth2Error),
    #[error("{0}")]
    OnLoginError(#[from] OnLoginError),
    #[error("{0}")]
    OnLogoutError(#[from] OnLogoutError),
}

impl UserError for OAuth2FinishFlowUrlError {
    fn user_error(&self) -> String {
        match self {
            Self::OAuth2Error(err) => err.user_error(),
            Self::OnLoginError(err) => err.user_error(),
            Self::OnLogoutError(err) => err.user_error(),
        }
    }
}
