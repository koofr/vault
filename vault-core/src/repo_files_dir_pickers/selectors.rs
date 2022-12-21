use crate::dir_pickers::selectors as dir_pickers_selectors;
use crate::dir_pickers::state::{DirPicker, DirPickerItem, DirPickerItemType};
use crate::repo_files::errors::{CreateDirError, RepoFilesErrors};
use crate::repo_files::selectors as repo_files_selectors;
use crate::repo_files::state::{RepoFile, RepoFileType};
use crate::store;

use super::state::Options;

pub fn select_options(state: &store::State, picker_id: u32) -> Option<Options> {
    dir_pickers_selectors::select_options(state, picker_id)
}

pub fn select_repo_id(state: &store::State, picker_id: u32) -> Option<String> {
    select_options(state, picker_id).map(|options| options.repo_id)
}

pub fn select_items(state: &store::State, picker: &DirPicker) -> Vec<DirPickerItem> {
    let mut items: Vec<DirPickerItem> = Vec::new();

    if let Some(repo_id) = select_repo_id(state, picker.id) {
        let root_file_id = repo_files_selectors::get_file_id(&repo_id, "/");

        if let Some(root_file) = repo_files_selectors::select_file(state, &root_file_id) {
            select_items_visit_file(state, picker, &mut items, root_file, 0);
        }
    }

    items
}

fn select_items_children<'a>(state: &'a store::State, file_id: &str) -> Option<Vec<&'a RepoFile>> {
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

    let id = file.id.clone();
    let is_root = match file.decrypted_path() {
        Ok("/") => true,
        _ => false,
    };
    let name = match repo_files_selectors::select_file_name(state, file) {
        Some(name) => name,
        None => {
            return;
        }
    };
    let typ = if is_root {
        DirPickerItemType::Repo
    } else {
        DirPickerItemType::Folder
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
                select_items_visit_file(state, picker, items, child, depth + 1);
            }
        }
    }
}

pub fn select_selected_file<'a>(state: &'a store::State, picker_id: u32) -> Option<&'a RepoFile> {
    dir_pickers_selectors::select_selected_file_id(state, picker_id)
        .and_then(|file_id| repo_files_selectors::select_file(state, file_id))
}

pub fn select_can_show_create_dir(state: &store::State, picker_id: u32) -> bool {
    select_selected_file(state, picker_id).is_some()
}

pub fn select_check_create_dir(
    state: &store::State,
    picker_id: u32,
    name: &str,
) -> Result<(), CreateDirError> {
    let parent_file =
        select_selected_file(state, picker_id).ok_or_else(RepoFilesErrors::not_found)?;

    let parent_path = parent_file.decrypted_path()?;

    repo_files_selectors::select_check_new_name_valid(
        state,
        &parent_file.repo_id,
        parent_path,
        name,
    )?;

    Ok(())
}
