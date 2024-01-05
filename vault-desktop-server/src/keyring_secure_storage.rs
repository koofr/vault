use std::collections::HashMap;

use keyring;

use vault_core::secure_storage::{MemorySecureStorage, SecureStorage};

#[derive(Debug)]
pub struct KeyringSecureStorage {
    service: String,
    name: String,
    storage: MemorySecureStorage,
}

impl KeyringSecureStorage {
    pub fn new(service: String, name: String) -> Result<Self, String> {
        let data = Self::load_data(&service, &name)?;

        let storage = MemorySecureStorage::new_with_data(data);

        Ok(Self {
            service,
            name,
            storage,
        })
    }

    pub fn load_data(service: &str, name: &str) -> Result<HashMap<String, String>, String> {
        let entry = keyring::Entry::new(service, name).map_err(|err| err.to_string())?;

        let data_json = match entry.get_password() {
            Ok(data_json) => data_json,
            Err(keyring::Error::NoEntry) => return Ok(HashMap::new()),
            Err(err) => return Err(err.to_string())?,
        };

        serde_json::from_str(&data_json).map_err(|err| err.to_string())
    }

    pub fn save_data(
        service: &str,
        name: &str,
        data: &HashMap<String, String>,
    ) -> Result<(), String> {
        let data_json = serde_json::to_string_pretty(&data).map_err(|err| err.to_string())?;

        let entry = keyring::Entry::new(service, name).map_err(|err| err.to_string())?;

        entry
            .set_password(&data_json)
            .map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let data = self.storage.get_data();

        Self::save_data(&self.service, &self.name, &data)
    }
}

impl SecureStorage for KeyringSecureStorage {
    fn get_item(&self, key: &str) -> Result<Option<String>, String> {
        self.storage.get_item(key)
    }

    fn set_item(&self, key: &str, value: &str) -> Result<(), String> {
        self.storage.set_item(key, value).unwrap();

        self.save()
    }

    fn remove_item(&self, key: &str) -> Result<(), String> {
        self.storage.remove_item(key).unwrap();

        self.save()
    }

    fn clear(&self) -> Result<(), String> {
        self.storage.clear().unwrap();

        self.save()
    }
}
