use thiserror::Error;

use crate::{http, secure_storage::errors::SecureStorageError, user_error::UserError};

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
    fn user_error(&self) -> String {
        match self {
            Self::InvalidOAuth2Token(err) => format!("Invalid OAuth 2 token: {}", err),
            Self::InvalidOAuth2State => "Invalid authentication state. Please try again.".into(),
            Self::InvalidGrant(err) => format!("Invalid authentication permissions: {}", err),
            Self::HttpError(err) => err.user_error(),
            Self::StorageError(err) => format!("Storage error: {}", err),
            Self::Unknown(err) => format!("Unknown error: {}", err),
        }
    }
}
