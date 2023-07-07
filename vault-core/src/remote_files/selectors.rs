use crate::{remote::RemoteError, store, utils::path_utils};

use super::{
    errors::RemoteFilesErrors,
    state::{
        Mount, MountOrigin, MountType, RemoteFile, RemoteFileExtType, RemoteFileType,
        RemoteFilesBreadcrumb,
    },
};

pub fn get_file_id(mount_id: &str, path: &str) -> String {
    format!("{}:{}", mount_id, path.to_lowercase())
}

pub fn get_file_unique_id(
    mount_id: &str,
    path: &str,
    size: i64,
    modified: i64,
    hash: Option<&str>,
) -> String {
    let digest = md5::compute(format!(
        "{}:{}:{}:{}",
        get_file_id(mount_id, path),
        size,
        modified,
        hash.unwrap_or(""),
    ));

    format!("{:x}", digest)
}

pub fn mount_origin_order(origin: &MountOrigin) -> u32 {
    match origin {
        MountOrigin::Hosted => 0,
        MountOrigin::Desktop => 1,
        MountOrigin::Dropbox => 2,
        MountOrigin::Googledrive => 3,
        MountOrigin::Onedrive => 4,
        MountOrigin::Share => 5,
        MountOrigin::Other { origin: _ } => 6,
    }
}

pub fn mount_sort_key<'a>(mount: &'a Mount) -> (u32, u32, &'a str) {
    (
        if mount.is_primary { 0 } else { 1 },
        mount_origin_order(&mount.origin),
        &mount.name_lower,
    )
}

pub fn mount_file_ext_type(mount: &Mount) -> RemoteFileExtType {
    match &mount.typ {
        MountType::Device => match &mount.origin {
            MountOrigin::Hosted => RemoteFileExtType::Hosted,
            MountOrigin::Desktop => {
                if mount.online {
                    RemoteFileExtType::Desktop
                } else {
                    RemoteFileExtType::DesktopOffline
                }
            }
            MountOrigin::Dropbox => RemoteFileExtType::Dropbox,
            MountOrigin::Googledrive => RemoteFileExtType::Googledrive,
            MountOrigin::Onedrive => RemoteFileExtType::Onedrive,
            MountOrigin::Share => RemoteFileExtType::Hosted,
            MountOrigin::Other { origin: _ } => RemoteFileExtType::Hosted,
        },
        MountType::Import => RemoteFileExtType::Import,
        MountType::Export => RemoteFileExtType::Export,
    }
}

pub fn select_mount<'a>(state: &'a store::State, mount_id: &str) -> Option<&'a Mount> {
    state.remote_files.mounts.get(mount_id)
}

pub fn remote_file_sort_key<'a>(file: &'a RemoteFile) -> (&'a RemoteFileType, &'a str) {
    (&file.typ, &file.name_lower)
}

pub fn select_children<'a>(state: &'a store::State, file_id: &str) -> Option<&'a Vec<String>> {
    state.remote_files.children.get(file_id)
}

pub fn select_files<'a>(
    state: &'a store::State,
    mount_id: &str,
    path: &str,
) -> Vec<&'a RemoteFile> {
    match select_children(state, &get_file_id(mount_id, path)) {
        Some(ids) => ids.iter().filter_map(|id| select_file(state, id)).collect(),
        None => vec![],
    }
}

pub fn select_file<'a>(state: &'a store::State, file_id: &str) -> Option<&'a RemoteFile> {
    state.remote_files.files.get(file_id)
}

pub fn select_file_name<'a>(state: &'a store::State, file: &'a RemoteFile) -> Option<&'a str> {
    if file.path == "/" {
        select_mount(state, &file.mount_id).map(|mount| mount.name.as_str())
    } else {
        Some(&file.name)
    }
}

pub fn check_name_valid(name: &str) -> Result<(), RemoteError> {
    if !name.is_empty() && !name.contains('/') {
        Ok(())
    } else {
        Err(RemoteFilesErrors::invalid_path())
    }
}

pub fn select_check_new_name_valid(
    state: &store::State,
    mount_id: &str,
    parent_path: &str,
    new_name: &str,
) -> Result<(), RemoteError> {
    check_name_valid(new_name)?;

    let new_path = path_utils::join_path_name(parent_path, new_name);

    match select_children(state, &get_file_id(mount_id, parent_path)) {
        Some(ids) => {
            if ids.contains(&get_file_id(mount_id, &new_path)) {
                Err(RemoteFilesErrors::already_exists())
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
}

pub fn select_breadcrumbs(
    state: &store::State,
    mount_id: &str,
    path: &str,
) -> Vec<RemoteFilesBreadcrumb> {
    let mount = match select_mount(state, mount_id) {
        Some(mount) => mount,
        None => {
            return vec![];
        }
    };

    let paths = path_utils::paths_chain(path);
    let paths_len = paths.len();

    paths
        .into_iter()
        .enumerate()
        .map(|(i, path)| {
            let id = get_file_id(mount_id, &path);
            let name = match path_utils::path_to_name(&path) {
                Some(name) => name.to_owned(),
                None => mount.name.clone(),
            };

            RemoteFilesBreadcrumb {
                id,
                mount_id: mount_id.to_owned(),
                path,
                name,
                last: i == paths_len - 1,
            }
        })
        .collect()
}
