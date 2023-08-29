use std::sync::Arc;

use serde::de::DeserializeOwned;

use super::{errors::SecureStorageError, secure_storage::SecureStorage};

pub struct SecureStorageService {
    secure_storage: Box<dyn SecureStorage + Send + Sync>,
}

impl SecureStorageService {
    pub fn new(secure_storage: Box<dyn SecureStorage + Send + Sync>) -> Self {
        Self { secure_storage }
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, SecureStorageError>
    where
        T: DeserializeOwned,
    {
        let raw_value = match self
            .secure_storage
            .get_item(key)
            .map_err(SecureStorageError::Error)?
        {
            Some(value) => value,
            None => {
                return Ok(None);
            }
        };

        Ok(Some(serde_json::from_str(&raw_value).map_err(|err| {
            SecureStorageError::SerializationError(Arc::new(err))
        })?))
    }

    pub fn set<T>(&self, key: &str, value: &T) -> Result<(), SecureStorageError>
    where
        T: serde::Serialize,
    {
        let raw_value = serde_json::to_string(value)
            .map_err(|err| SecureStorageError::SerializationError(Arc::new(err)))?;

        self.secure_storage
            .set_item(key, &raw_value)
            .map_err(SecureStorageError::Error)?;

        Ok(())
    }

    pub fn remove(&self, key: &str) -> Result<(), SecureStorageError> {
        self.secure_storage
            .remove_item(key)
            .map_err(SecureStorageError::Error)
    }

    pub fn clear(&self) -> Result<(), SecureStorageError> {
        self.secure_storage
            .clear()
            .map_err(SecureStorageError::Error)
    }
}
