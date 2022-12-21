use vault_core::secure_storage::SecureStorage;

pub struct BrowserSecureStorage {
    local_storage: web_sys::Storage,
}

unsafe impl Send for BrowserSecureStorage {}
unsafe impl Sync for BrowserSecureStorage {}

impl BrowserSecureStorage {
    pub fn new() -> Self {
        Self {
            local_storage: web_sys::window().unwrap().local_storage().unwrap().unwrap(),
        }
    }
}

impl SecureStorage for BrowserSecureStorage {
    fn get_item(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.local_storage.get_item(key).unwrap_or(None))
    }

    fn set_item(&self, key: &str, value: &str) -> Result<(), String> {
        let _ = self.local_storage.set_item(key, value);

        Ok(())
    }

    fn remove_item(&self, key: &str) -> Result<(), String> {
        let _ = self.local_storage.remove_item(key);

        Ok(())
    }
}
