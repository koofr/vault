use crate::{
    remote_files::state::{RemoteFile, RemoteFilesLocation},
    remote_files_dir_pickers::selectors as remote_files_dir_pickers_selectors,
    store,
};

use super::state::{RepoCreate, RepoCreateForm};

pub fn select_repo_create<'a>(state: &'a store::State, create_id: u32) -> Option<&'a RepoCreate> {
    state.repo_creates.creates.get(&create_id)
}

pub fn select_form<'a>(state: &'a store::State, create_id: u32) -> Option<&'a RepoCreateForm> {
    match select_repo_create(state, create_id) {
        Some(RepoCreate::Form(repo_create_form)) => Some(repo_create_form),
        _ => None,
    }
}

pub fn select_location<'a>(
    state: &'a store::State,
    create_id: u32,
) -> Option<&'a RemoteFilesLocation> {
    select_form(state, create_id).and_then(|form| form.location.as_ref())
}

pub fn select_primary_mount_location(
    state: &store::State,
    create_id: u32,
) -> Option<RemoteFilesLocation> {
    select_form(state, create_id).and_then(|form| {
        form.primary_mount_id
            .as_ref()
            .map(|mount_id| RemoteFilesLocation {
                mount_id: mount_id.to_owned(),
                path: String::from("/"),
            })
    })
}

pub fn select_location_dir_picker_id(state: &store::State, create_id: u32) -> Option<u32> {
    select_form(state, create_id).and_then(|form| form.location_dir_picker_id)
}

pub fn select_location_dir_picker_selected_file<'a>(
    state: &'a store::State,
    create_id: u32,
) -> Option<&'a RemoteFile> {
    select_location_dir_picker_id(state, create_id).and_then(|picker_id| {
        remote_files_dir_pickers_selectors::select_selected_file(state, picker_id)
    })
}

pub fn select_location_dir_picker_can_select(state: &store::State, create_id: u32) -> bool {
    select_location_dir_picker_selected_file(state, create_id)
        .filter(|file| file.path != "/")
        .is_some()
}

pub fn select_location_dir_picker_create_dir_enabled(state: &store::State, create_id: u32) -> bool {
    select_location_dir_picker_id(state, create_id)
        .map(|picker_id| {
            remote_files_dir_pickers_selectors::select_create_dir_enabled(state, picker_id)
        })
        .unwrap_or(false)
}

pub fn is_password_valid(password: &str) -> bool {
    password.len() >= 8
}

pub fn select_can_create(state: &store::State, create_id: u32) -> bool {
    match &select_form(state, create_id) {
        Some(RepoCreateForm {
            location, password, ..
        }) => location.is_some() && is_password_valid(password),
        _ => false,
    }
}
