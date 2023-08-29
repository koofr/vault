use vault_core::secure_storage::SecureStorage;

pub struct BrowserSecureStorage {
    storage: web_sys::Storage,
}

unsafe impl Send for BrowserSecureStorage {}
unsafe impl Sync for BrowserSecureStorage {}

impl BrowserSecureStorage {
    pub fn new(storage: web_sys::Storage) -> Self {
        Self { storage }
    }
}

impl SecureStorage for BrowserSecureStorage {
    fn get_item(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.storage.get_item(key).unwrap_or(None))
    }

    fn set_item(&self, key: &str, value: &str) -> Result<(), String> {
        let _ = self.storage.set_item(key, value);

        Ok(())
    }

    fn remove_item(&self, key: &str) -> Result<(), String> {
        let _ = self.storage.remove_item(key);

        Ok(())
    }

    fn clear(&self) -> Result<(), String> {
        let _ = self.storage.remove_item("vaultOAuth2Token");
        let _ = self.storage.remove_item("vaultOAuth2State");
        let _ = self.storage.remove_item("vaultLoginRedirect");

        Ok(())
    }
}
