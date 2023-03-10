use crate::{cipher::Cipher, remote::test_helpers as remote_test_helpers, utils::path_utils};

use super::{mutations::decrypt_files_list_recursive_item, state::RepoFilesListRecursiveItem};

pub fn create_list_recursive_item_dir(
    mount_id: &str,
    remote_repo_path: &str,
    repo_id: &str,
    root_path: &str,
    item_path: &str,
    cipher: &Cipher,
) -> RepoFilesListRecursiveItem {
    let encrypted_item_path = cipher.encrypt_path(item_path);
    let encrypted_item_name = path_utils::split_parent_name(&encrypted_item_path)
        .map(|(_, name)| name)
        .unwrap_or("");
    let remote_item = remote_test_helpers::create_files_list_recursive_item_dir(
        &encrypted_item_path,
        encrypted_item_name,
    );
    decrypt_files_list_recursive_item(
        mount_id,
        &path_utils::join_paths(remote_repo_path, &cipher.encrypt_path(root_path)),
        repo_id,
        root_path,
        remote_item,
        &cipher,
    )
}

pub fn create_list_recursive_item_file(
    mount_id: &str,
    remote_repo_path: &str,
    repo_id: &str,
    root_path: &str,
    item_path: &str,
    cipher: &Cipher,
) -> RepoFilesListRecursiveItem {
    let encrypted_item_path = cipher.encrypt_path(item_path);
    let encrypted_item_name = path_utils::split_parent_name(&encrypted_item_path)
        .map(|(_, name)| name)
        .unwrap_or("");
    let remote_item = remote_test_helpers::create_files_list_recursive_item_file(
        &encrypted_item_path,
        encrypted_item_name,
    );
    decrypt_files_list_recursive_item(
        mount_id,
        &path_utils::join_paths(remote_repo_path, &cipher.encrypt_path(root_path)),
        repo_id,
        root_path,
        remote_item,
        &cipher,
    )
}
