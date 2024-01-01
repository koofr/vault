use thiserror::Error;

use std::sync::Arc;

use crate::{
    remote::RemoteError,
    repos::errors::{GetCipherError, RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone)]
pub enum RepoFileTagsDecodeError {
    #[error("{0}")]
    Base64Error(#[from] data_encoding::DecodeError),
    #[error("{0}")]
    DecryptError(Arc<std::io::Error>),
    #[error("{0}")]
    RMPSerdeError(Arc<rmp_serde::decode::Error>),
}

impl PartialEq for RepoFileTagsDecodeError {
    fn eq(&self, unknown: &Self) -> bool {
        self.to_string() == unknown.to_string()
    }
}

impl UserError for RepoFileTagsDecodeError {
    fn user_error(&self) -> String {
        match self {
            Self::Base64Error(err) => format!("Failed to base64 decode tags: {}", err.to_string()),
            Self::DecryptError(err) => format!("Failed to decrypt tags: {}", err.to_string()),
            Self::RMPSerdeError(err) => format!("Failed to deserialize tags: {}", err.to_string()),
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum RepoFileTagsEncodeError {
    #[error("{0}")]
    RMPSerdeError(Arc<rmp_serde::encode::Error>),
    #[error("{0}")]
    EncryptError(Arc<std::io::Error>),
}

impl PartialEq for RepoFileTagsEncodeError {
    fn eq(&self, unknown: &Self) -> bool {
        self.to_string() == unknown.to_string()
    }
}

impl UserError for RepoFileTagsEncodeError {
    fn user_error(&self) -> String {
        match self {
            Self::RMPSerdeError(err) => format!("Failed to serialize tags: {}", err.to_string()),
            Self::EncryptError(err) => format!("Failed to encrypt tags: {}", err.to_string()),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptTagsError {
    #[error("multiple values")]
    MultipleValues,
    #[error("encrypted hash mismatch: {expected_encrypted_hash:?} != {encrypted_hash:?}")]
    EncryptedHashMismatch {
        expected_encrypted_hash: Option<String>,
        encrypted_hash: Option<String>,
    },
    #[error("{0}")]
    DecodeError(#[from] RepoFileTagsDecodeError),
}

impl UserError for DecryptTagsError {
    fn user_error(&self) -> String {
        match self {
            Self::MultipleValues => format!("Multiple tags values found"),
            Self::EncryptedHashMismatch {
                expected_encrypted_hash,
                encrypted_hash,
            } => format!(
                "Encrypted file hash does not match the encrypted hash in tags: {:?} != {:?}",
                expected_encrypted_hash, encrypted_hash
            ),
            Self::DecodeError(err) => err.user_error(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SetTagsError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("file not found")]
    FileNotFound,
    #[error("missing encrypted hash")]
    MissingEncryptedHash,
    #[error("invalid encrypted hash")]
    InvalidEncryptedHash(hex::FromHexError),
    #[error("encrypted hash mismatch: {expected_encrypted_hash} != {encrypted_hash:?}")]
    EncryptedHashMismatch {
        expected_encrypted_hash: String,
        encrypted_hash: Option<String>,
    },
    #[error("{0}")]
    RepoFileTagsEncodeError(#[from] RepoFileTagsEncodeError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

// impl UserError for SetTagsError {
//     fn user_error(&self) -> String {
//         match self {
//             Self::RepoNotFound(err) => err.user_error(),
//             Self::RepoLocked(err) => err.user_error(),
//             Self::FileNotFound => self.to_string(),
//             Self::FilesEmpty => self.to_string(),
//             Self::DecryptFilenameError(err) => err.user_error(),
//             Self::DecryptSizeError(err) => err.user_error(),
//             Self::RemoteError(err) => err.user_error(),
//             Self::IOError(err) => err.to_string(),
//             Self::Aborted => self.to_string(),
//         }
//     }
// }

impl From<GetCipherError> for SetTagsError {
    fn from(err: GetCipherError) -> Self {
        match err {
            GetCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            GetCipherError::RepoLocked(err) => Self::RepoLocked(err),
        }
    }
}
