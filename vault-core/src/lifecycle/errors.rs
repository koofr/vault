use thiserror::Error;

use crate::{oauth2::errors::OAuth2Error, remote::RemoteError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadError {
    #[error("{0}")]
    OAuth2Error(#[from] OAuth2Error),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for LoadError {
    fn user_error(&self) -> String {
        match self {
            Self::OAuth2Error(err) => err.user_error(),
            Self::RemoteError(err) => err.user_error(),
        }
    }
}
