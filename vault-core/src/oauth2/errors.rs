use thiserror::Error;

use crate::{http, user_error::UserError};

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
    #[error("{0}")]
    Unknown(String),
}
