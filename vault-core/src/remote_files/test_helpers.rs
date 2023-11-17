use crate::{
    remote::test_helpers as remote_test_helpers,
    types::{MountId, RemotePath},
    utils::path_utils,
};

use super::{mutations::files_file_to_remote_file, selectors, state::RemoteFile};

pub fn create_file(mount_id: &str, path: &str) -> RemoteFile {
    files_file_to_remote_file(
        selectors::get_file_id(
            &MountId(mount_id.to_owned()),
            &RemotePath(path.to_owned()).to_lowercase(),
        ),
        MountId(mount_id.to_owned()),
        RemotePath(path.to_owned()),
        remote_test_helpers::create_file(path_utils::path_to_name(path).unwrap_or("")),
    )
}

pub fn create_dir(mount_id: &str, path: &str) -> RemoteFile {
    files_file_to_remote_file(
        selectors::get_file_id(
            &MountId(mount_id.to_owned()),
            &RemotePath(path.to_owned()).to_lowercase(),
        ),
        MountId(mount_id.to_owned()),
        RemotePath(path.to_owned()),
        remote_test_helpers::create_dir(path_utils::path_to_name(path).unwrap_or("")),
    )
}
