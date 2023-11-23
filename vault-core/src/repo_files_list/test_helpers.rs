use crate::{
    cipher::Cipher,
    remote::test_helpers as remote_test_helpers,
    types::{DecryptedPath, MountId, RemotePath, RepoId},
    utils::{path_utils, remote_path_utils},
};

use super::{mutations::decrypt_files_list_recursive_item, state::RepoFilesListRecursiveItem};

pub fn create_list_recursive_item_dir(
    mount_id: &str,
    remote_repo_path: &str,
    repo_id: &str,
    root_path: &str,
    item_path: &str,
    cipher: &Cipher,
) -> RepoFilesListRecursiveItem {
    let encrypted_root_path = cipher.encrypt_path(&DecryptedPath(root_path.to_owned()));
    let encrypted_item_path = cipher.encrypt_path(&DecryptedPath(item_path.to_owned()));
    let encrypted_item_name = path_utils::split_parent_name(&encrypted_item_path.0)
        .map(|(_, name)| name)
        .unwrap_or("");
    let remote_item = remote_test_helpers::create_files_list_recursive_item_dir(
        &encrypted_item_path.0,
        encrypted_item_name,
    );
    decrypt_files_list_recursive_item(
        &MountId(mount_id.to_owned()),
        &remote_path_utils::join_paths(
            &RemotePath(remote_repo_path.to_owned()),
            &RemotePath(cipher.encrypt_path(&DecryptedPath(root_path.to_owned())).0),
        ),
        &RepoId(repo_id.to_owned()),
        &encrypted_root_path,
        &Ok(DecryptedPath(root_path.to_owned())),
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
    let encrypted_root_path = cipher.encrypt_path(&DecryptedPath(root_path.to_owned()));
    let encrypted_item_path = cipher.encrypt_path(&DecryptedPath(item_path.to_owned()));
    let encrypted_item_name = path_utils::split_parent_name(&encrypted_item_path.0)
        .map(|(_, name)| name)
        .unwrap_or("");
    let remote_item = remote_test_helpers::create_files_list_recursive_item_file(
        &encrypted_item_path.0,
        encrypted_item_name,
    );
    decrypt_files_list_recursive_item(
        &MountId(mount_id.to_owned()),
        &remote_path_utils::join_paths(
            &RemotePath(remote_repo_path.to_owned()),
            &RemotePath(cipher.encrypt_path(&DecryptedPath(root_path.to_owned())).0),
        ),
        &RepoId(repo_id.to_owned()),
        &encrypted_root_path,
        &Ok(DecryptedPath(root_path.to_owned())),
        remote_item,
        &cipher,
    )
}
