use rand_core;
use thiserror::Error;

#[derive(Debug, Error)]
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
    #[error("generate nonce error: {0:?}")]
    GenerateNonceError(rand_core::Error),
}

impl From<rand_core::Error> for CipherError {
    fn from(err: rand_core::Error) -> Self {
        Self::GenerateNonceError(err)
    }
}

impl Into<std::io::Error> for CipherError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, self)
    }
}

#[derive(Debug, Error, Clone)]
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
}
