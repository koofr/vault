use thiserror::Error;

use crate::{common::errors::InvalidNameError, intl, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptSizeError {
    #[error("{0}")]
    DecryptSizeError(#[from] vault_crypto::errors::DecryptSizeError),
}

impl UserError for DecryptSizeError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::DecryptSizeError(err) => intl::format_message!(
                intl_service,
                "core.cipher.decrypt_size.error",
                "Error shown when decrypting an encrypted file size fails.",
                "Failed to decrypt size: {error}",
                &[("error", intl::FormatValue::String(&err.to_string()))]
            ),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        use vault_crypto::errors::DecryptFilenameError;

        match self {
            Self::DecryptFilenameError(DecryptFilenameError::DecodeError(_)) => {
                intl::format_message!(
                    intl_service,
                    "core.cipher.decrypt_filename_decode.error",
                    "Error shown when decoding an encrypted file name fails.",
                    "Failed to decode file name"
                )
            }
            Self::DecryptFilenameError(DecryptFilenameError::DecryptError) => {
                intl::format_message!(
                    intl_service,
                    "core.cipher.decrypt_filename_decrypt.error",
                    "Error shown when decrypting a file name fails.",
                    "Failed to decrypt file name. Vault files can only be uploaded using Vault apps or rclone. If all your files have errors please check that you've used the correct salt."
                )
            }
            Self::DecryptFilenameError(DecryptFilenameError::UnicodeError(_)) => {
                intl::format_message!(
                    intl_service,
                    "core.cipher.decrypt_filename_unicode.error",
                    "Error shown when a decrypted file name is not valid Unicode text.",
                    "File name is not a valid Unicode text"
                )
            }
            Self::InvalidNameError(err) => err.user_error(intl_service),
        }
    }
}
