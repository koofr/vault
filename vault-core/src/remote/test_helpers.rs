use std::collections::HashMap;

use crate::utils::path_utils;

use super::models;

pub fn create_file(name: &str) -> models::FilesFile {
    models::FilesFile {
        name: name.to_owned(),
        typ: String::from("file"),
        modified: 1,
        size: 100,
        content_type: String::from("text/plain"),
        hash: Some(String::from("hash")),
        tags: HashMap::new(),
    }
}

pub fn create_dir(name: &str) -> models::FilesFile {
    models::FilesFile {
        name: name.to_owned(),
        typ: String::from("dir"),
        modified: 1,
        size: 0,
        content_type: String::from(""),
        hash: None,
        tags: HashMap::new(),
    }
}

pub fn create_repo(repo_id: &str, mount_id: &str, path: &str) -> models::VaultRepo {
    models::VaultRepo {
        id: repo_id.to_owned(),
        name: path_utils::path_to_name(path).unwrap_or("/").to_owned(),
        mount_id: mount_id.to_owned(),
        path: path.to_owned(),
        salt: None,
        password_validator: String::from("pv"),
        password_validator_encrypted: String::from("pve"),
        added: 1,
    }
}

pub fn create_files_list_recursive_item_file(
    path: &str,
    name: &str,
) -> models::FilesListRecursiveItem {
    models::FilesListRecursiveItem::File {
        path: path.to_owned(),
        file: create_file(name),
    }
}

pub fn create_files_list_recursive_item_dir(
    path: &str,
    name: &str,
) -> models::FilesListRecursiveItem {
    models::FilesListRecursiveItem::File {
        path: path.to_owned(),
        file: create_dir(name),
    }
}

pub fn create_files_list_recursive_item_error(
    path: Option<&str>,
    error: models::ApiErrorDetails,
) -> models::FilesListRecursiveItem {
    models::FilesListRecursiveItem::Error {
        path: path.map(|path| path.to_owned()),
        error,
    }
}
