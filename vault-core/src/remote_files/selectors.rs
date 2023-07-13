use crate::{
    files::file_category::{ext_to_file_category, FileCategory},
    remote::RemoteError,
    store,
    utils::{name_utils, path_utils},
};

use super::{
    errors::RemoteFilesErrors,
    state::{Mount, RemoteFile, RemoteFileType, RemoteFilesBreadcrumb},
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

pub fn get_file_ext_category<'a>(name_lower: &'a str) -> (Option<String>, FileCategory) {
    let ext = name_utils::name_to_ext(name_lower);

    (
        ext.map(str::to_string),
        ext.and_then(ext_to_file_category)
            .unwrap_or(FileCategory::Generic),
    )
}

pub fn mount_sort_key<'a>(mount: &'a Mount) -> (u32, u32, &'a str) {
    (
        if mount.is_primary { 0 } else { 1 },
        mount.origin.order(),
        &mount.name_lower,
    )
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

pub fn select_is_root_loaded(state: &store::State, mount_id: &str, path: &str) -> bool {
    state
        .remote_files
        .loaded_roots
        .contains(&get_file_id(&mount_id, &path))
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
            select_file(state, &get_file_id(&mount.id, "/")).map(|file| (mount, file))
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
    mount_id: &'a str,
    path: &'a str,
) -> Vec<&'a RemoteFile> {
    select_children(state, &get_file_id(mount_id, path))
        .map(|ids| ids.iter().filter_map(|id| select_file(state, id)).collect())
        .unwrap_or_else(|| vec![])
}
