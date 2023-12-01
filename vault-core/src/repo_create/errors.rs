use thiserror::Error;

use crate::{remote::RemoteError, repos::errors::LoadReposError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CreateLoadError {
    #[error("{0}")]
    LoadReposError(#[from] LoadReposError),
    #[error("{0}")]
    LoadPrimaryMountError(RemoteError),
}

impl UserError for CreateLoadError {
    fn user_error(&self) -> String {
        match self {
            Self::LoadReposError(err) => err.user_error(),
            Self::LoadPrimaryMountError(err) => err.user_error(),
        }
    }
}
