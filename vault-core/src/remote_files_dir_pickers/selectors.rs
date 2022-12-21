use crate::dir_pickers::selectors as dir_pickers_selectors;
use crate::dir_pickers::state::{DirPicker, DirPickerItem, DirPickerItemType};
use crate::remote::RemoteError;
use crate::remote_files::errors::RemoteFilesErrors;
use crate::remote_files::selectors as remote_files_selectors;
use crate::remote_files::state::{
    MountOrigin, MountType, RemoteFile, RemoteFileExtType, RemoteFileType,
};
use crate::store;

use super::state::Options;

pub fn remote_file_ext_type_to_dir_picker_item_type(typ: &RemoteFileExtType) -> DirPickerItemType {
    match typ {
        RemoteFileExtType::Folder => DirPickerItemType::Folder,
        RemoteFileExtType::Import => DirPickerItemType::Import,
        RemoteFileExtType::Export => DirPickerItemType::Export,
        RemoteFileExtType::Hosted => DirPickerItemType::Hosted,
        RemoteFileExtType::Desktop => DirPickerItemType::Desktop,
        RemoteFileExtType::DesktopOffline => DirPickerItemType::DesktopOffline,
        RemoteFileExtType::Dropbox => DirPickerItemType::Dropbox,
        RemoteFileExtType::Googledrive => DirPickerItemType::Googledrive,
        RemoteFileExtType::Onedrive => DirPickerItemType::Onedrive,
    }
}

pub const BOOKMARKS_ITEM_ID: &'static str = "bookmarks";
pub const PLACES_ITEM_ID: &'static str = "places";
pub const SHARED_ITEM_ID: &'static str = "shared";

pub const BOOKMARKS_ITEM_ID_PREFIX: &'static str = "bookmarks:";
pub const PLACES_ITEM_ID_PREFIX: &'static str = "places:";
pub const SHARED_ITEM_ID_PREFIX: &'static str = "shared:";

pub fn get_bookmark_id(file_id: &str) -> String {
    format!("{}{}", BOOKMARKS_ITEM_ID_PREFIX, file_id)
}

pub fn get_places_id(file_id: &str) -> String {
    format!("{}{}", PLACES_ITEM_ID_PREFIX, file_id)
}

pub fn get_shared_id(file_id: &str) -> String {
    format!("{}{}", SHARED_ITEM_ID_PREFIX, file_id)
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
                    BOOKMARKS_ITEM_ID_PREFIX,
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
        let file_id = remote_files_selectors::get_file_id(&mount.id, "/");
        if let Some(file) = remote_files_selectors::select_file(state, &file_id) {
            select_items_visit_file(
                state,
                picker,
                items,
                file,
                PLACES_ITEM_ID_PREFIX,
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
                    SHARED_ITEM_ID_PREFIX,
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
    id_prefix: &str,
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

    let id = format!("{}{}", id_prefix, &file.id);
    let is_root = file.path == "/";
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
                remote_file_ext_type_to_dir_picker_item_type(
                    &remote_files_selectors::mount_file_ext_type(mount),
                )
            } else {
                DirPickerItemType::Folder
            }
        }
    };
    let is_open = picker.open_ids.contains(&id);
    let is_selected = picker.selected_id.as_deref() == Some(&id);
    let is_loading = picker.loading_ids.contains(&id);
    let has_arrow = depth > 0
        && match &children {
            Some(children) => !children.is_empty(),
            None => true,
        };
    let spaces = depth - if has_arrow { 1 } else { 0 };

    items.push(DirPickerItem {
        id,
        file_id: Some(file.id.clone()),
        typ,
        is_open,
        is_selected,
        is_selectable: true,
        is_loading,
        spaces,
        has_arrow,
        text: name.to_owned(),
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
    dir_pickers_selectors::select_selected_file_id(state, picker_id)
        .and_then(|file_id| remote_files_selectors::select_file(state, file_id))
}

pub fn select_can_show_create_dir(state: &store::State, picker_id: u32) -> bool {
    select_selected_file(state, picker_id).is_some()
}

pub fn select_check_create_dir(
    state: &store::State,
    picker_id: u32,
    name: &str,
) -> Result<(), RemoteError> {
    let parent_file =
        select_selected_file(state, picker_id).ok_or_else(RemoteFilesErrors::not_found)?;

    remote_files_selectors::select_check_new_name_valid(
        state,
        &parent_file.mount_id,
        &parent_file.path,
        name,
    )
}
