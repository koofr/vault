use vault_core::secure_storage::{MemorySecureStorage, SecureStorage};

use crate::{
    data_path::get_data_path,
    file_secure_storage::{get_file_secure_storage_path, FileSecureStorage},
    keyring_secure_storage::KeyringSecureStorage,
};

pub fn init_keyring_secure_storage(
    app_id: &str,
) -> (Box<dyn SecureStorage + Send + Sync>, Option<String>) {
    match KeyringSecureStorage::new(app_id.to_owned(), "storage".to_owned()) {
        Ok(secure_storage) => (Box::new(secure_storage), None),
        Err(err) => {
            let secure_storage = Box::new(MemorySecureStorage::new());

            (
                secure_storage,
                Some(format!("Failed to load app data: {}", err.to_string())),
            )
        }
    }
}

pub fn init_file_secure_storage(
    app_id: &str,
) -> (Box<dyn SecureStorage + Send + Sync>, Option<String>) {
    let data_path = match get_data_path(&app_id) {
        Ok(data_path) => data_path,
        Err(err) => {
            let secure_storage = Box::new(MemorySecureStorage::new());

            return (
                secure_storage,
                Some(format!(
                    "Failed to initialize app data path: {}",
                    err.to_string()
                )),
            );
        }
    };

    log::info!("App data path: {:?}", data_path);

    let secure_storage_path = get_file_secure_storage_path(data_path);

    match FileSecureStorage::new(secure_storage_path) {
        Ok(secure_storage) => (Box::new(secure_storage), None),
        Err(err) => {
            let secure_storage = Box::new(MemorySecureStorage::new());

            (
                secure_storage,
                Some(format!("Failed to load app data: {}", err.to_string())),
            )
        }
    }
}
