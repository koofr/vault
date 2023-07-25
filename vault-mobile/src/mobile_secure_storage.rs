use crate::SecureStorage;

pub struct MobileSecureStorage {
    secure_storage: Box<dyn SecureStorage>,
}

impl MobileSecureStorage {
    pub fn new(secure_storage: Box<dyn SecureStorage>) -> Self {
        Self { secure_storage }
    }
}

impl vault_core::secure_storage::SecureStorage for MobileSecureStorage {
    fn get_item(&self, key: &str) -> Result<Option<String>, String> {
        self.secure_storage
            .get_item(key.to_owned())
            .map_err(|e| e.to_string())
    }

    fn set_item(&self, key: &str, value: &str) -> Result<(), String> {
        self.secure_storage
            .set_item(key.to_owned(), value.to_owned())
            .map_err(|e| e.to_string())
    }

    fn remove_item(&self, key: &str) -> Result<(), String> {
        self.secure_storage
            .remove_item(key.to_owned())
            .map_err(|e| e.to_string())
    }

    fn clear(&self) -> Result<(), String> {
        self.secure_storage.clear().map_err(|e| e.to_string())
    }
}
