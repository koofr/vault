use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SecureStorageError {
    #[error("serialization error: {0}")]
    SerializationError(Arc<serde_json::Error>),
    #[error("secure storage error: {0}")]
    Error(String),
}

impl PartialEq for SecureStorageError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
