use lazy_static::lazy_static;

use crate::{
    dir_pickers::{
        selectors as dir_pickers_selectors,
        state::{DirPicker, DirPickerFileId, DirPickerItem, DirPickerItemId, DirPickerItemType},
    },
    remote::RemoteError,
    remote_files::{
        errors::RemoteFilesErrors,
        selectors as remote_files_selectors,
        state::{MountOrigin, MountType, RemoteFile, RemoteFileExtType, RemoteFileType},
    },
    store,
    types::{RemoteFileId, RemoteNameLower, REMOTE_PATH_LOWER_ROOT},
};

use super::state::Options;

lazy_static! {
    pub static ref BOOKMARKS_ITEM_ID: DirPickerItemId = DirPickerItemId("bookmarks".into());
    pub static ref PLACES_ITEM_ID: DirPickerItemId = DirPickerItemId("places".into());
    pub static ref SHARED_ITEM_ID: DirPickerItemId = DirPickerItemId("shared".into());
    pub static ref BOOKMARKS_ITEM_ID_PREFIX: DirPickerItemId = DirPickerItemId("bookmarks:".into());
    pub static ref PLACES_ITEM_ID_PREFIX: DirPickerItemId = DirPickerItemId("places:".into());
    pub static ref SHARED_ITEM_ID_PREFIX: DirPickerItemId = DirPickerItemId("shared:".into());
}

pub fn get_bookmark_id(file_id: &str) -> DirPickerItemId {
    DirPickerItemId(format!("{}{}", BOOKMARKS_ITEM_ID_PREFIX.0, file_id))
}

pub fn get_places_id(file_id: &RemoteFileId) -> DirPickerItemId {
    DirPickerItemId(format!("{}{}", PLACES_ITEM_ID_PREFIX.0, file_id.0))
}

pub fn get_shared_id(file_id: &RemoteFileId) -> DirPickerItemId {
    DirPickerItemId(format!("{}{}", SHARED_ITEM_ID_PREFIX.0, file_id.0))
}

pub fn get_options(picker: &DirPicker) -> Options {
    dir_pickers_selectors::select_picker_options(picker)
}

pub fn select_items(state: &store::State, picker: &DirPicker) -> Vec<DirPickerItem> {
    let options = get_options(picker);

    let mut items: Vec<DirPickerItem> = Vec::new();

    select_items_bookmarks(state, picker, &mut items, &options);
    select_items_places(state, picker, &mut items, &options);
    select_items_shared(state, picker, &mut items, &options);

    items
}

pub fn select_items_bookmarks(
    state: &store::State,
    picker: &DirPicker,
    items: &mut Vec<DirPickerItem>,
    options: &Options,
) {
    let bookmarks: Vec<&RemoteFile> = state
        .remote_files
        .bookmark_file_ids
        .iter()
        .filter_map(|id| remote_files_selectors::select_file(state, id))
        .collect();

    if bookmarks.len() > 0 {
        let id = BOOKMARKS_ITEM_ID.to_owned();
        let is_open = picker.open_ids.contains(&id);
        let is_loading = picker.loading_ids.contains(&id);

        items.push(DirPickerItem {
            id,
            file_id: None,
            typ: DirPickerItemType::Bookmarks,
            is_open,
            is_selected: false,
            is_selectable: false,
            is_loading,
            spaces: 0,
            has_arrow: false,
            text: String::from("Bookmarks"),
        });

        if is_open {
            for bookmark_file in bookmarks {
                select_items_visit_file(
                    state,
                    picker,
                    items,
                    bookmark_file,
                    &BOOKMARKS_ITEM_ID_PREFIX,
                    1,
                    Some(DirPickerItemType::Bookmark),
                    options,
                );
            }
        }
    }
}

pub fn select_items_places(
    state: &store::State,
    picker: &DirPicker,
    items: &mut Vec<DirPickerItem>,
    options: &Options,
) {
    for mount in state
        .remote_files
        .online_place_mount_ids
        .iter()
        .filter_map(|mount_id| state.remote_files.mounts.get(mount_id))
    {
        let file_id = remote_files_selectors::get_file_id(&mount.id, &REMOTE_PATH_LOWER_ROOT);
        if let Some(file) = remote_files_selectors::select_file(state, &file_id) {
            select_items_visit_file(
                state,
                picker,
                items,
                file,
                &PLACES_ITEM_ID_PREFIX,
                0,
                None,
                options,
            );
        }
    }
}

pub fn select_items_shared(
    state: &store::State,
    picker: &DirPicker,
    items: &mut Vec<DirPickerItem>,
    options: &Options,
) {
    if options.only_hosted_devices {
        return;
    }

    let shared: Vec<&RemoteFile> = state
        .remote_files
        .shared_file_ids
        .iter()
        .filter_map(|id| remote_files_selectors::select_file(state, id))
        .collect();

    if shared.len() > 0 {
        let id = SHARED_ITEM_ID.to_owned();
        let is_open = picker.open_ids.contains(&id);
        let is_loading = picker.loading_ids.contains(&id);

        items.push(DirPickerItem {
            id,
            file_id: None,
            typ: DirPickerItemType::Shared,
            is_open,
            is_selected: false,
            is_selectable: false,
            is_loading,
            spaces: 0,
            has_arrow: false,
            text: String::from("Shared"),
        });

        if is_open {
            for shared_file in shared {
                select_items_visit_file(
                    state,
                    picker,
                    items,
                    shared_file,
                    &SHARED_ITEM_ID_PREFIX,
                    1,
                    None,
                    options,
                );
            }
        }
    }
}

fn select_items_visit_file(
    state: &store::State,
    picker: &DirPicker,
    items: &mut Vec<DirPickerItem>,
    file: &RemoteFile,
    id_prefix: &DirPickerItemId,
    depth: u16,
    override_type: Option<DirPickerItemType>,
    options: &Options,
) {
    let children: Option<Vec<&RemoteFile>> =
        remote_files_selectors::select_children(state, &file.id).map(|ids| {
            ids.iter()
                .filter_map(|id| remote_files_selectors::select_file(state, id))
                .filter(|file| file.typ == RemoteFileType::Dir)
                .collect()
        });

    let id = DirPickerItemId(format!("{}{}", id_prefix.0, file.id.0));
    let is_root = file.path.is_root();
    let name = match remote_files_selectors::select_file_name(state, &file) {
        Some(name) => name,
        None => {
            return;
        }
    };
    let mount = match remote_files_selectors::select_mount(state, &file.mount_id) {
        Some(mount) => mount,
        None => {
            return;
        }
    };
    if options.only_hosted_devices {
        if mount.typ != MountType::Device || mount.origin != MountOrigin::Hosted {
            return;
        }
    }
    let typ = match override_type {
        Some(typ) => typ,
        None => {
            if is_root {
                RemoteFileExtType::from(mount).into()
            } else {
                DirPickerItemType::Folder
            }
        }
    };
    let is_open = picker.open_ids.contains(&id);
    let is_selected = picker.selected_id.as_ref() == Some(&id);
    let is_loading = picker.loading_ids.contains(&id);
    let has_arrow = depth > 0
        && match &children {
            Some(children) => !children.is_empty(),
            None => true,
        };
    let spaces = depth - if has_arrow { 1 } else { 0 };

    items.push(DirPickerItem {
        id,
        file_id: Some(DirPickerFileId(file.id.0.clone())),
        typ,
        is_open,
        is_selected,
        is_selectable: true,
        is_loading,
        spaces,
        has_arrow,
        text: name.0.clone(),
    });

    if is_open {
        if let Some(children) = children {
            for child in children {
                select_items_visit_file(
                    state,
                    picker,
                    items,
                    child,
                    id_prefix,
                    depth + 1,
                    None,
                    options,
                );
            }
        }
    }
}

pub fn select_selected_file<'a>(state: &'a store::State, picker_id: u32) -> Option<&'a RemoteFile> {
    dir_pickers_selectors::select_selected_file_id(state, picker_id).and_then(|file_id| {
        remote_files_selectors::select_file(state, &RemoteFileId(file_id.0.clone()))
    })
}

pub fn select_create_dir_enabled(state: &store::State, picker_id: u32) -> bool {
    select_selected_file(state, picker_id).is_some()
}

pub fn select_check_create_dir(
    state: &store::State,
    picker_id: u32,
    name: &RemoteNameLower,
) -> Result<(), RemoteError> {
    let parent_file =
        select_selected_file(state, picker_id).ok_or_else(RemoteFilesErrors::not_found)?;

    remote_files_selectors::select_check_new_name_valid(
        state,
        &parent_file.mount_id,
        &parent_file.path.to_lowercase(),
        name,
    )
}
