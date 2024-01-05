use std::{collections::HashMap, fs, path::PathBuf};

use vault_core::secure_storage::{MemorySecureStorage, SecureStorage};

pub fn get_file_secure_storage_path(data_path: PathBuf) -> PathBuf {
    data_path.join("storage.json")
}

#[derive(Debug)]
pub struct FileSecureStorage {
    path: PathBuf,
    storage: MemorySecureStorage,
}

impl FileSecureStorage {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let data = Self::load_data(&path)?;

        let storage = MemorySecureStorage::new_with_data(data);

        Ok(Self { path, storage })
    }

    pub fn load_data(path: &PathBuf) -> Result<HashMap<String, String>, String> {
        let data_json = match fs::read_to_string(path) {
            Ok(data_json) => data_json,
            Err(err) if matches!(err.kind(), std::io::ErrorKind::NotFound) => {
                return Ok(HashMap::new())
            }
            Err(err) => return Err(err.to_string())?,
        };

        serde_json::from_str(&data_json).map_err(|err| err.to_string())
    }

    pub fn save_data(path: &PathBuf, data: &HashMap<String, String>) -> Result<(), String> {
        let data_json = serde_json::to_string_pretty(&data).map_err(|err| err.to_string())?;

        fs::write(path, data_json).map_err(|err| err.to_string())?;

        Ok(())
    }

    pub fn save(&self) -> Result<(), String> {
        let data = self.storage.get_data();

        Self::save_data(&self.path, &data)
    }
}

impl SecureStorage for FileSecureStorage {
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
