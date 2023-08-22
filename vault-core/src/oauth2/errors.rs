use thiserror::Error;

use crate::{http, secure_storage::errors::SecureStorageError, user_error::UserError};

#[derive(Error, Debug, Clone, UserError)]
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
