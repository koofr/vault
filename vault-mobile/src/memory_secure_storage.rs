use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{SecureStorage, SecureStorageError};

#[derive(Debug)]
pub struct MemorySecureStorage {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl MemorySecureStorage {
    pub fn new() -> Self {
        Self::new_with_data(HashMap::new())
    }

    pub fn new_with_data(data: HashMap<String, String>) -> Self {
        Self {
            data: Arc::new(RwLock::new(data)),
        }
    }

    pub fn get_data(&self) -> RwLockReadGuard<'_, HashMap<String, String>> {
        self.data.read().unwrap()
    }
}

impl SecureStorage for MemorySecureStorage {
    fn get_item(&self, key: String) -> Result<Option<String>, SecureStorageError> {
        Ok(self.data.read().unwrap().get(&key).cloned())
    }

    fn set_item(&self, key: String, value: String) -> Result<(), SecureStorageError> {
        self.data.write().unwrap().insert(key, value);

        Ok(())
    }

    fn remove_item(&self, key: String) -> Result<(), SecureStorageError> {
        self.data.write().unwrap().remove(&key);

        Ok(())
    }

    fn clear(&self) -> Result<(), SecureStorageError> {
        self.data.write().unwrap().clear();

        Ok(())
    }
}
