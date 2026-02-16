use thiserror::Error;

use crate::{
    intl, oauth2::errors::OAuth2Error, remote::RemoteError, repos::errors::LoadReposError,
    secure_storage::errors::SecureStorageError, user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadError {
    #[error("{0}")]
    OAuth2LoadError(OAuth2Error),
    #[error("{0}")]
    OnLoginError(OnLoginError),
}

impl UserError for LoadError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::OAuth2LoadError(err) => err.user_error(intl_service),
            Self::OnLoginError(err) => err.user_error(intl_service),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OnLoginError {
    #[error("{0}")]
    LoadUserError(RemoteError),
    #[error("{0}")]
    LoadReposError(LoadReposError),
    #[error("{0}")]
    LoadSpaceUsageError(RemoteError),
}

impl UserError for OnLoginError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::LoadUserError(err) => intl::format_message!(
                intl_service,
                "core.lifecycle.on_login_load_user.error",
                "Error shown after login when loading the user profile fails.",
                "Failed to load user: {error}",
                &[(
                    "error",
                    intl::FormatValue::String(&err.user_error(intl_service))
                )]
            ),
            Self::LoadReposError(err) => intl::format_message!(
                intl_service,
                "core.lifecycle.on_login_load_repos.error",
                "Error shown after login when loading the Safe Boxes fails.",
                "Failed to load safe boxes: {error}",
                &[(
                    "error",
                    intl::FormatValue::String(&err.user_error(intl_service))
                )]
            ),
            Self::LoadSpaceUsageError(err) => intl::format_message!(
                intl_service,
                "core.lifecycle.on_login_load_space_usage.error",
                "Error shown after login when loading the storage space usage fails.",
                "Failed to load space usage: {error}",
                &[(
                    "error",
                    intl::FormatValue::String(&err.user_error(intl_service))
                )]
            ),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::OAuth2LogoutError(err) => err.user_error(intl_service),
            Self::OnLogoutError(err) => err.user_error(intl_service),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OnLogoutError {
    #[error("{0}")]
    ClearStorageError(SecureStorageError),
}

impl UserError for OnLogoutError {
    fn user_error(&self, _intl_service: &intl::IntlService) -> String {
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::OAuth2Error(err) => err.user_error(intl_service),
            Self::OnLoginError(err) => err.user_error(intl_service),
            Self::OnLogoutError(err) => err.user_error(intl_service),
        }
    }
}
