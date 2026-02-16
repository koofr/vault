use thiserror::Error;

use crate::{http, intl, secure_storage::errors::SecureStorageError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum OAuth2Error {
    #[error("invalid oauth2 token: {0}")]
    InvalidOAuth2Token(String),
    #[error("invalid oauth2 state")]
    InvalidOAuth2State,
    #[error("{0}")]
    InvalidGrant(String),
    #[error("{0}")]
    HttpError(#[from] http::HttpError),
    #[error("storage error: {0}")]
    StorageError(#[from] SecureStorageError),
    #[error("{0}")]
    Unknown(String),
}

impl UserError for OAuth2Error {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::InvalidOAuth2Token(err) => intl::format_message!(
                intl_service,
                "core.oauth2.invalid_oauth2_token.error",
                "Error shown when the OAuth token returned from login is invalid.",
                "Invalid OAuth 2 token: {error}",
                &[("error", intl::FormatValue::String(err))]
            ),
            Self::InvalidOAuth2State => intl::format_message!(
                intl_service,
                "core.oauth2.invalid_oauth2_state.error",
                "Error shown when OAuth state validation fails during login.",
                "Invalid authentication state. Please try again."
            ),
            Self::InvalidGrant(err) => intl::format_message!(
                intl_service,
                "core.oauth2.invalid_grant.error",
                "Error shown when OAuth permissions or grant is invalid or denied during login.",
                "Invalid authentication permissions: {error}",
                &[("error", intl::FormatValue::String(err))]
            ),
            Self::HttpError(err) => err.user_error(intl_service),
            Self::StorageError(err) => err.user_error(intl_service),
            Self::Unknown(err) => intl::format_message!(
                intl_service,
                "core.oauth2.unknown.error",
                "Error shown for unexpected OAuth login issues.",
                "Unknown error: {error}",
                &[("error", intl::FormatValue::String(err))]
            ),
        }
    }
}
