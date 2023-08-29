use std::sync::Arc;

use vault_core::secure_storage::{MemorySecureStorage, SecureStorage};

pub struct TestSecureStorage {
    pub get_item_fn: Box<dyn Fn(&str) -> Result<Option<String>, String> + Send + Sync + 'static>,
    pub set_item_fn: Box<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync + 'static>,
    pub remove_item_fn: Box<dyn Fn(&str) -> Result<(), String> + Send + Sync + 'static>,
    pub clear_fn: Box<dyn Fn() -> Result<(), String> + Send + Sync + 'static>,
}

impl TestSecureStorage {
    pub fn wrap(secure_storage: Box<dyn SecureStorage + Send + Sync>) -> Self {
        let secure_storage = Arc::new(secure_storage);

        let get_item_fn_secure_storage = secure_storage.clone();
        let set_item_fn_secure_storage = secure_storage.clone();
        let remove_item_fn_secure_storage = secure_storage.clone();
        let clear_fn_secure_storage = secure_storage.clone();

        Self {
            get_item_fn: Box::new(move |key| get_item_fn_secure_storage.get_item(key)),
            set_item_fn: Box::new(move |key, value| {
                set_item_fn_secure_storage.set_item(key, value)
            }),
            remove_item_fn: Box::new(move |key| remove_item_fn_secure_storage.remove_item(key)),
            clear_fn: Box::new(move || clear_fn_secure_storage.clear()),
        }
    }

    pub fn wrap_memory() -> Self {
        Self::wrap(Box::new(MemorySecureStorage::new()))
    }
}

impl SecureStorage for TestSecureStorage {
    fn get_item(&self, key: &str) -> Result<Option<String>, String> {
        (self.get_item_fn)(key)
    }

    fn set_item(&self, key: &str, value: &str) -> Result<(), String> {
        (self.set_item_fn)(key, value)
    }

    fn remove_item(&self, key: &str) -> Result<(), String> {
        (self.remove_item_fn)(key)
    }

    fn clear(&self) -> Result<(), String> {
        (self.clear_fn)()
    }
}
