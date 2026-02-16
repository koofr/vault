use std::sync::Arc;

use thiserror::Error;

use crate::{intl, user_error::UserError};

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

impl UserError for SecureStorageError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.secure_storage.secure_storage.error",
            "Error shown when secure storage (keychain/keystore) operations fail.",
            "Storage error: {error}",
            &[("error", intl::FormatValue::String(&self.to_string()))]
        )
    }
}
