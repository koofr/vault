use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecureStorageError {
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("secure storage error: {0}")]
    Error(String),
}
