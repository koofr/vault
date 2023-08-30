use thiserror::Error;

use crate::{oauth2::errors::OAuth2Error, remote::RemoteError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum LoadError {
    #[error("{0}")]
    OAuth2Error(#[from] OAuth2Error),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}
