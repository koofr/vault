use crate::{
    dir_pickers::{
        selectors as dir_pickers_selectors,
        state::{DirPicker, DirPickerFileId, DirPickerItem, DirPickerItemId, DirPickerItemType},
    },
    repo_files::{
        selectors as repo_files_selectors,
        state::{RepoFile, RepoFileType},
    },
    store,
    types::{RepoFileId, RepoId, DECRYPTED_PATH_ROOT},
};

use super::state::Options;

pub fn select_options(state: &store::State, picker_id: u32) -> Option<Options> {
    dir_pickers_selectors::select_options(state, picker_id)
}

pub fn select_repo_id(state: &store::State, picker_id: u32) -> Option<RepoId> {
    select_options(state, picker_id).map(|options| options.repo_id)
}

pub fn select_items(state: &store::State, picker: &DirPicker) -> Vec<DirPickerItem> {
    let mut items: Vec<DirPickerItem> = Vec::new();

    if let Some(repo_id) = select_repo_id(state, picker.id) {
        let root_file_id = repo_files_selectors::get_file_id(&repo_id, &DECRYPTED_PATH_ROOT);

        if let Some(root_file) = repo_files_selectors::select_file(state, &root_file_id) {
            select_items_visit_file(state, picker, &mut items, root_file, 0);
        }
    }

    items
}

fn select_items_children<'a>(
    state: &'a store::State,
    file_id: &RepoFileId,
) -> Option<Vec<&'a RepoFile>> {
    repo_files_selectors::select_children(state, file_id).map(|ids| {
        ids.iter()
            .filter_map(|id| repo_files_selectors::select_file(state, id))
            .filter(|file| file.typ == RepoFileType::Dir)
            .collect()
    })
}

fn select_items_visit_file(
    state: &store::State,
    picker: &DirPicker,
    items: &mut Vec<DirPickerItem>,
    file: &RepoFile,
    depth: u16,
) {
    let children = select_items_children(state, &file.id);

    let id = DirPickerItemId(file.id.0.clone());
    let is_root = match file.decrypted_path() {
        Ok(path) if path.is_root() => true,
        _ => false,
    };
    let name = match repo_files_selectors::select_file_name(state, file) {
        Ok(name) => name,
        Err(_) => {
            return;
        }
    };
    let typ = if is_root {
        DirPickerItemType::Repo
    } else {
        DirPickerItemType::Folder
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
        text: name.0.to_owned(),
    });

    if is_open {
        if let Some(children) = children {
            for child in children {
                select_items_visit_file(state, picker, items, child, depth + 1);
            }
        }
    }
}
