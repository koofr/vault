use std::collections::HashMap;

use crate::{
    types::{MountId, RemoteName, RemotePath, RepoId},
    utils::path_utils,
};

use super::models;

pub fn create_file(name: &str) -> models::FilesFile {
    models::FilesFile {
        name: RemoteName(name.to_owned()),
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
        name: RemoteName(name.to_owned()),
        typ: String::from("dir"),
        modified: 1,
        size: 0,
        content_type: String::from(""),
        hash: None,
        tags: HashMap::new(),
    }
}

pub fn files_file_to_bundle_file(file: models::FilesFile) -> models::BundleFile {
    models::BundleFile {
        name: file.name,
        typ: file.typ,
        modified: file.modified,
        size: file.size,
        content_type: file.content_type,
        hash: file.hash,
        tags: file.tags,
    }
}

pub fn create_bundle(root_name: &str, files: Option<Vec<models::FilesFile>>) -> models::Bundle {
    models::Bundle {
        file: files_file_to_bundle_file(create_dir(root_name)),
        files: files.map(|files| files.into_iter().map(files_file_to_bundle_file).collect()),
    }
}

pub fn create_repo(repo_id: &str, mount_id: &str, path: &str) -> models::VaultRepo {
    models::VaultRepo {
        id: RepoId(repo_id.to_owned()),
        name: path_utils::path_to_name(path).unwrap_or("/").to_owned(),
        mount_id: MountId(mount_id.to_owned()),
        path: RemotePath(path.to_owned()),
        salt: Some("salt".into()),
        password_validator: String::from("a8668309-60f9-40f1-9a4c-0d1de0ff5852"),
        password_validator_encrypted: String::from("v2:UkNMT05FAADWjQahYq7E1ij2zegBBHbFuDbGIHAvdpym3P4eW2CPQcWhcTuAz4YGLAwRQzj2PoP4vwS2hAEwFwqMlFsWTgLMQ2ONzdNJK4d3kaVw"),
        added: 1,
    }
}

pub fn create_files_list_recursive_item_file(
    path: &str,
    name: &str,
) -> models::FilesListRecursiveItem {
    models::FilesListRecursiveItem::File {
        path: RemotePath(path.to_owned()),
        file: create_file(name),
    }
}

pub fn create_files_list_recursive_item_dir(
    path: &str,
    name: &str,
) -> models::FilesListRecursiveItem {
    models::FilesListRecursiveItem::File {
        path: RemotePath(path.to_owned()),
        file: create_dir(name),
    }
}

pub fn create_files_list_recursive_item_error(
    path: Option<&str>,
    error: models::ApiErrorDetails,
) -> models::FilesListRecursiveItem {
    models::FilesListRecursiveItem::Error {
        path: path.map(ToOwned::to_owned).map(RemotePath),
        error,
    }
}
