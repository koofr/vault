use crate::{
    files::file_category::{ext_to_file_category, FileCategory},
    remote::RemoteError,
    store,
    types::{
        MountId, RemoteFileId, RemoteName, RemoteNameLower, RemotePath, RemotePathLower,
        REMOTE_PATH_LOWER_ROOT,
    },
    utils::{name_utils, remote_path_utils},
};

use super::{
    errors::RemoteFilesErrors,
    state::{Mount, RemoteFile, RemoteFileType, RemoteFilesBreadcrumb},
};

pub fn get_file_id(mount_id: &MountId, path: &RemotePathLower) -> RemoteFileId {
    RemoteFileId(format!("{}:{}", mount_id.0, path.0))
}

pub fn get_file_unique_id(
    mount_id: &MountId,
    path: &RemotePathLower,
    size: Option<i64>,
    modified: Option<i64>,
    hash: Option<&str>,
) -> String {
    let digest = md5::compute(format!(
        "{}:{}:{}:{}",
        get_file_id(mount_id, path).0,
        size.unwrap_or(0),
        modified.unwrap_or(0),
        hash.unwrap_or(""),
    ));

    format!("{:x}", digest)
}

pub fn get_file_ext_category(name_lower: &RemoteNameLower) -> (Option<String>, FileCategory) {
    let ext = name_utils::name_to_ext(&name_lower.0);

    (
        ext.map(str::to_string),
        ext.and_then(ext_to_file_category)
            .unwrap_or(FileCategory::Generic),
    )
}

pub fn mount_sort_key<'a>(mount: &'a Mount) -> (u32, u32, &'a RemoteNameLower) {
    (
        if mount.is_primary { 0 } else { 1 },
        mount.origin.order(),
        &mount.name_lower,
    )
}

pub fn select_mount<'a>(state: &'a store::State, mount_id: &MountId) -> Option<&'a Mount> {
    state.remote_files.mounts.get(mount_id)
}

pub fn remote_file_sort_key<'a>(file: &'a RemoteFile) -> (&'a RemoteFileType, &RemoteNameLower) {
    (&file.typ, &file.name_lower)
}

pub fn select_children<'a>(
    state: &'a store::State,
    file_id: &RemoteFileId,
) -> Option<&'a Vec<RemoteFileId>> {
    state.remote_files.children.get(file_id)
}

pub fn select_files<'a>(
    state: &'a store::State,
    mount_id: &MountId,
    path: &RemotePathLower,
) -> Vec<&'a RemoteFile> {
    match select_children(state, &get_file_id(mount_id, path)) {
        Some(ids) => ids.iter().filter_map(|id| select_file(state, id)).collect(),
        None => vec![],
    }
}

pub fn select_file<'a>(state: &'a store::State, file_id: &RemoteFileId) -> Option<&'a RemoteFile> {
    state.remote_files.files.get(file_id)
}

pub fn select_file_name<'a>(
    state: &'a store::State,
    file: &'a RemoteFile,
) -> Option<&'a RemoteName> {
    if file.path.is_root() {
        select_mount(state, &file.mount_id).map(|mount| &mount.name)
    } else {
        Some(&file.name)
    }
}

pub fn select_is_root_loaded(
    state: &store::State,
    mount_id: &MountId,
    path: &RemotePathLower,
) -> bool {
    state
        .remote_files
        .loaded_roots
        .contains(&get_file_id(&mount_id, &path))
}

pub fn check_name_valid(name: &RemoteNameLower) -> Result<(), RemoteError> {
    if !name.0.is_empty() && !name.0.contains('/') {
        Ok(())
    } else {
        Err(RemoteFilesErrors::invalid_path())
    }
}

pub fn select_check_new_name_valid(
    state: &store::State,
    mount_id: &MountId,
    parent_path: &RemotePathLower,
    new_name: &RemoteNameLower,
) -> Result<(), RemoteError> {
    check_name_valid(new_name)?;

    let new_path = remote_path_utils::join_path_name_lower(parent_path, new_name);

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
    mount_id: &MountId,
    path: &RemotePath,
) -> Vec<RemoteFilesBreadcrumb> {
    let mount = match select_mount(state, mount_id) {
        Some(mount) => mount,
        None => {
            return vec![];
        }
    };

    let paths = remote_path_utils::paths_chain(path);
    let paths_len = paths.len();

    paths
        .into_iter()
        .enumerate()
        .map(|(i, path)| {
            let id = get_file_id(mount_id, &path.to_lowercase());
            let name = match remote_path_utils::path_to_name(&path) {
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

pub fn select_bookmarks_files(state: &store::State) -> Vec<&RemoteFile> {
    state
        .remote_files
        .bookmark_file_ids
        .iter()
        .filter_map(|id| select_file(state, id))
        .collect()
}

pub fn select_places_mount_files(state: &store::State) -> Vec<(&Mount, &RemoteFile)> {
    state
        .remote_files
        .online_place_mount_ids
        .iter()
        .filter_map(|mount_id| state.remote_files.mounts.get(mount_id))
        .filter_map(|mount| {
            select_file(state, &get_file_id(&mount.id, &REMOTE_PATH_LOWER_ROOT))
                .map(|file| (mount, file))
        })
        .collect()
}

pub fn select_shared_mount_files(state: &store::State) -> Vec<(&Mount, &RemoteFile)> {
    state
        .remote_files
        .shared_file_ids
        .iter()
        .filter_map(|id| {
            select_file(state, id).and_then(|file| {
                state
                    .remote_files
                    .mounts
                    .get(&file.mount_id)
                    .map(|mount| (mount, file))
            })
        })
        .collect()
}

pub fn select_storage_files<'a>(
    state: &'a store::State,
    mount_id: &'a MountId,
    path: &'a RemotePathLower,
) -> Vec<&'a RemoteFile> {
    select_children(state, &get_file_id(mount_id, path))
        .map(|ids| ids.iter().filter_map(|id| select_file(state, id)).collect())
        .unwrap_or_else(|| vec![])
}
