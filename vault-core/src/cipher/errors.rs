use thiserror::Error;

use crate::{common::errors::InvalidNameError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptSizeError {
    #[error("{0}")]
    DecryptSizeError(#[from] vault_crypto::errors::DecryptSizeError),
}

impl UserError for DecryptSizeError {
    fn user_error(&self) -> String {
        match self {
            Self::DecryptSizeError(err) => format!("Failed to decrypt size: {}", err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptFilenameError {
    #[error("{0}")]
    DecryptFilenameError(#[from] vault_crypto::errors::DecryptFilenameError),
    #[error("{0}")]
    InvalidNameError(#[from] InvalidNameError),
}

impl UserError for DecryptFilenameError {
    fn user_error(&self) -> String {
        use vault_crypto::errors::DecryptFilenameError;

        match self {
            Self::DecryptFilenameError(DecryptFilenameError::DecodeError(_)) => "Failed to decode file name".into(),
            Self::DecryptFilenameError(DecryptFilenameError::DecryptError) => "Failed to decrypt file name. Vault files can only be uploaded using Vault apps or rclone. If all your files have errors please check that you've used the correct salt.".into(),
            Self::DecryptFilenameError(DecryptFilenameError::UnicodeError(_)) => "File name is not a valid Unicode text".into(),
            Self::InvalidNameError(err) => err.user_error(),
        }
    }
}
