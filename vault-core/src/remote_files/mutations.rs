use crate::remote::models;
use crate::store;
use crate::utils::path_utils;

use super::selectors;
use super::state::{Mount, MountType, RemoteFile, RemoteFileType};

pub fn mount_to_remote_file(id: String, mount_id: String) -> RemoteFile {
    RemoteFile {
        id,
        mount_id,
        path: String::from("/"),
        name: String::from(""),
        name_lower: String::from(""),
        typ: RemoteFileType::Dir,
        size: 0,
        modified: 0,
    }
}

pub fn files_file_to_remote_file(
    id: String,
    mount_id: String,
    path: String,
    file: models::FilesFile,
) -> RemoteFile {
    let name_lower = file.name.to_lowercase();

    RemoteFile {
        id,
        mount_id,
        path,
        name: file.name,
        name_lower,
        typ: file.typ.as_str().into(),
        size: file.size,
        modified: file.modified,
    }
}

pub fn bundle_file_to_remote_file(
    id: String,
    mount_id: String,
    path: String,
    file: models::BundleFile,
) -> RemoteFile {
    let name_lower = file.name.to_lowercase();

    RemoteFile {
        id,
        mount_id,
        path,
        name: file.name,
        name_lower,
        typ: file.typ.as_str().into(),
        size: file.size,
        modified: file.modified,
    }
}

fn bookmark_to_remote_file(id: String, bookmark: models::Bookmark) -> RemoteFile {
    let name_lower = bookmark.name.to_lowercase();

    RemoteFile {
        id,
        mount_id: bookmark.mount_id,
        path: bookmark.path,
        name: bookmark.name,
        name_lower,
        typ: RemoteFileType::Dir,
        size: 0,
        modified: 0,
    }
}

fn shared_file_to_remote_file(id: String, shared_file: models::SharedFile) -> RemoteFile {
    RemoteFile {
        id,
        mount_id: shared_file.mount.id,
        path: String::from("/"),
        name: shared_file.name.clone(),
        name_lower: shared_file.name.to_lowercase(),
        typ: shared_file.typ.as_str().into(),
        size: shared_file.size,
        modified: shared_file.modified,
    }
}

fn dir_to_remote_file(id: String, mount_id: String, path: String) -> RemoteFile {
    let name = path_utils::path_to_name(&path)
        .map(|name| name.to_owned())
        .unwrap_or(String::from(""));
    let name_lower = name.to_lowercase();

    RemoteFile {
        id,
        mount_id,
        path,
        name,
        name_lower,
        typ: RemoteFileType::Dir,
        size: 0,
        modified: 0,
    }
}

pub fn mount_loaded(state: &mut store::State, mount: models::Mount) {
    state
        .remote_files
        .mounts
        .insert(mount.id.clone(), mount.into());
}

pub fn place_loaded(state: &mut store::State, mount: models::Mount) {
    let file_id = selectors::get_file_id(&mount.id, "/");

    state.remote_files.files.insert(
        file_id.clone(),
        mount_to_remote_file(file_id.clone(), mount.id.clone()),
    );

    mount_loaded(state, mount);
}

pub fn places_loaded(state: &mut store::State, mounts: Vec<models::Mount>) {
    // TODO remove all old places
    for mount in mounts {
        place_loaded(state, mount);
    }

    let mut place_mounts: Vec<&Mount> = state
        .remote_files
        .mounts
        .values()
        .filter(|mount| mount.typ == MountType::Device)
        .collect();

    place_mounts.sort_by(|x, y| selectors::mount_sort_key(&x).cmp(&selectors::mount_sort_key(&y)));

    state.remote_files.place_mount_ids =
        place_mounts.iter().map(|mount| mount.id.clone()).collect();

    state.remote_files.online_place_mount_ids = place_mounts
        .iter()
        .filter(|mount| mount.online)
        .map(|mount| mount.id.clone())
        .collect();
}

pub fn bookmark_loaded(state: &mut store::State, bookmark: models::Bookmark) {
    let file_id = selectors::get_file_id(&bookmark.mount_id, &bookmark.path);

    state.remote_files.files.insert(
        file_id.clone(),
        bookmark_to_remote_file(file_id.clone(), bookmark),
    );

    state.remote_files.bookmark_file_ids.push(file_id);
}

pub fn bookmarks_loaded(state: &mut store::State, bookmarks: Vec<models::Bookmark>) {
    state.remote_files.bookmark_file_ids.clear();

    for bookmark in bookmarks {
        bookmark_loaded(state, bookmark);
    }
}

pub fn shared_file_loaded(state: &mut store::State, shared_file: models::SharedFile) {
    let file_id = selectors::get_file_id(&shared_file.mount.id, "/");

    mount_loaded(state, shared_file.mount.clone());

    state.remote_files.files.insert(
        file_id.clone(),
        shared_file_to_remote_file(file_id.clone(), shared_file),
    );

    state.remote_files.shared_file_ids.push(file_id);
}

pub fn shared_files_loaded(state: &mut store::State, shared_files: Vec<models::SharedFile>) {
    state.remote_files.shared_file_ids.clear();

    for shared_file in shared_files {
        shared_file_loaded(state, shared_file);
    }
}

pub fn sort_children(state: &mut store::State, file_id: &str) {
    if let Some(children_ids) = state.remote_files.children.get(file_id) {
        let mut children: Vec<&RemoteFile> = children_ids
            .iter()
            .filter_map(|id| state.remote_files.files.get(id))
            .collect();

        children.sort_by(|x, y| {
            selectors::remote_file_sort_key(x).cmp(&selectors::remote_file_sort_key(y))
        });

        let children_ids: Vec<String> = children.iter().map(|file| file.id.clone()).collect();

        state
            .remote_files
            .children
            .insert(file_id.to_owned(), children_ids);
    }
}

pub fn bundle_loaded(state: &mut store::State, mount_id: &str, path: &str, bundle: models::Bundle) {
    let root_file_id = selectors::get_file_id(mount_id, path);

    let models::Bundle {
        file: bundle_file,
        files: bundle_files,
    } = bundle;

    state.remote_files.files.insert(
        root_file_id.clone(),
        bundle_file_to_remote_file(
            root_file_id.clone(),
            mount_id.to_owned(),
            path.to_owned(),
            bundle_file,
        ),
    );

    if let Some(files) = bundle_files {
        let mut children = Vec::with_capacity(files.len());

        for file in files {
            let file_path = path_utils::join_path_name(path, &file.name);
            let file_id = selectors::get_file_id(mount_id, &file_path);
            let remote_file = bundle_file_to_remote_file(
                file_id.clone(),
                mount_id.to_owned(),
                file_path.clone(),
                file,
            );

            children.push(file_id.clone());

            state.remote_files.files.insert(file_id, remote_file);
        }

        state
            .remote_files
            .children
            .insert(root_file_id.clone(), children);

        sort_children(state, &root_file_id);
    }

    state.remote_files.loaded_roots.insert(root_file_id.clone());
}

pub fn add_child(state: &mut store::State, parent_id: &str, child_id: String) {
    if let Some(children) = state.remote_files.children.get_mut(parent_id) {
        if !children.contains(&child_id) {
            children.push(child_id);

            sort_children(state, &parent_id);
        }
    }
}

pub fn dir_created(state: &mut store::State, mount_id: &str, path: &str) {
    let file_id = selectors::get_file_id(mount_id, path);

    state.remote_files.files.insert(
        file_id.clone(),
        dir_to_remote_file(file_id.clone(), mount_id.to_owned(), path.to_owned()),
    );

    if let Some(parent_path) = path_utils::parent_path(path) {
        let parent_id = selectors::get_file_id(mount_id, &parent_path);

        add_child(state, &parent_id, file_id);
    }
}

pub fn file_created(state: &mut store::State, mount_id: &str, path: &str, file: models::FilesFile) {
    let file_id = selectors::get_file_id(mount_id, path);

    state.remote_files.files.insert(
        file_id.clone(),
        files_file_to_remote_file(file_id.clone(), mount_id.to_owned(), path.to_owned(), file),
    );

    if let Some(parent_path) = path_utils::parent_path(path) {
        let parent_id = selectors::get_file_id(mount_id, &parent_path);

        add_child(state, &parent_id, file_id);
    }
}

pub fn remove_child(state: &mut store::State, parent_id: &str, child_id: &str) {
    if let Some(children) = state.remote_files.children.get_mut(parent_id) {
        children.retain(|id| id != &child_id);
    }
}

pub fn file_removed(state: &mut store::State, mount_id: &str, path: &str) {
    let file_id = selectors::get_file_id(mount_id, path);

    if let Some(parent_path) = path_utils::parent_path(path) {
        let parent_id = selectors::get_file_id(mount_id, &parent_path);

        remove_child(state, &parent_id, &file_id);
    }

    cleanup_file(state, &file_id);
}

pub fn cleanup_file(state: &mut store::State, file_id: &str) {
    state.remote_files.files.remove(file_id);

    let file_id_prefix = if file_id.ends_with('/') {
        file_id.to_owned()
    } else {
        format!("{file_id}/")
    };

    state
        .remote_files
        .files
        .retain(|file_id, _| !file_id.starts_with(&file_id_prefix));

    state.remote_files.children.remove(file_id);

    state
        .remote_files
        .children
        .retain(|file_id, _| !file_id.starts_with(&file_id_prefix));
}

pub fn file_copied(
    state: &mut store::State,
    mount_id: &str,
    new_path: &str,
    new_file: models::FilesFile,
) {
    let new_file_id = selectors::get_file_id(mount_id, new_path);
    let new_parent_path = match path_utils::parent_path(new_path) {
        Some(new_parent_path) => new_parent_path,
        None => {
            return;
        }
    };
    let new_parent_id = selectors::get_file_id(mount_id, new_parent_path);

    state.remote_files.files.insert(
        new_file_id.clone(),
        files_file_to_remote_file(
            new_file_id.clone(),
            mount_id.to_owned(),
            new_path.to_owned(),
            new_file,
        ),
    );

    add_child(state, &new_parent_id, new_file_id.clone());
}

pub fn file_moved(
    state: &mut store::State,
    mount_id: &str,
    old_path: &str,
    new_path: &str,
    new_file: models::FilesFile,
) {
    let old_file_id = selectors::get_file_id(mount_id, old_path);
    let old_parent_path = match path_utils::parent_path(old_path) {
        Some(old_parent_path) => old_parent_path,
        None => {
            return;
        }
    };
    let old_parent_id = selectors::get_file_id(mount_id, old_parent_path);

    let new_file_id = selectors::get_file_id(mount_id, new_path);
    let new_parent_path = match path_utils::parent_path(new_path) {
        Some(new_parent_path) => new_parent_path,
        None => {
            return;
        }
    };
    let new_parent_id = selectors::get_file_id(mount_id, new_parent_path);

    if let Some(_) = state.remote_files.files.remove(&old_file_id) {
        file_children_change_parent_path(state, &old_file_id, new_path);
    }

    state.remote_files.files.insert(
        new_file_id.clone(),
        files_file_to_remote_file(
            new_file_id.clone(),
            mount_id.to_owned(),
            new_path.to_owned(),
            new_file,
        ),
    );

    remove_child(state, &old_parent_id, &old_file_id);

    // TODO ensure new parent path

    add_child(state, &new_parent_id, new_file_id.clone());
}

pub fn file_children_change_parent_path(
    state: &mut store::State,
    file_id: &str,
    new_parent_path: &str,
) {
    if let Some(old_children_ids) = state
        .remote_files
        .children
        .get(file_id)
        .map(|ids| ids.clone())
    {
        let new_children_ids: Vec<String> = Vec::with_capacity(old_children_ids.len());

        for old_child_id in &old_children_ids {
            if let Some(mut child) = state.remote_files.files.remove(old_child_id) {
                let new_child_path = path_utils::join_path_name(new_parent_path, &child.name);
                let new_child_id = selectors::get_file_id(&child.mount_id, &new_child_path);

                file_children_change_parent_path(state, old_child_id, &new_child_path);

                child.id = new_child_id.clone();
                child.path = new_child_path.clone();

                state.remote_files.files.insert(new_child_id.clone(), child);
            }
        }

        state
            .remote_files
            .children
            .insert(file_id.to_owned(), new_children_ids);
    }
}
