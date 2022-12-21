use thiserror::Error;

use crate::oauth2::errors::OAuth2Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("oauth2 error: {0}")]
    OAuth2Error(#[from] OAuth2Error),
}
