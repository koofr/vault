use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SecureStorageError {
    #[error("serialization error: {0}")]
    SerializationError(Arc<serde_json::Error>),
    #[error("secure storage error: {0}")]
    Error(String),
}
