use std::sync::Arc;

use rand_core;
use thiserror::Error;

use crate::common::errors::InvalidNameError;

#[derive(Error, Debug, Clone)]
#[error("generate nonce error: {0:?}")]
pub struct GenerateNonceError(pub Arc<rand_core::Error>);

impl PartialEq for GenerateNonceError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CipherError {
    #[error("file is too short to be decrypted")]
    EncryptedFileTooShort,
    #[error("file has truncated block header")]
    EncryptedFileBadHeader,
    #[error("not an encrypted file - bad magic string")]
    EncryptedBadMagic,
    #[error("encryption error")]
    EncryptionError,
    #[error("decryption error")]
    DecryptionError,
    #[error("{0}")]
    GenerateNonceError(GenerateNonceError),
}

impl From<rand_core::Error> for CipherError {
    fn from(err: rand_core::Error) -> Self {
        Self::GenerateNonceError(GenerateNonceError(Arc::new(err)))
    }
}

impl Into<std::io::Error> for CipherError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, self)
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptSizeError {
    #[error("file is too short to be decrypted")]
    EncryptedFileTooShort,
    #[error("file has truncated block header")]
    EncryptedFileBadHeader,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptFilenameError {
    #[error("decode error: {0}")]
    DecodeError(String),
    #[error("decrypt error")]
    DecryptError,
    #[error("unicode error: {0}")]
    UnicodeError(String),
    #[error("{0}")]
    InvalidName(#[from] InvalidNameError),
}
