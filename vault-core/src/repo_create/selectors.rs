use crate::{
    remote::RemoteError,
    remote_files::{
        errors::RemoteFilesErrors,
        state::{RemoteFile, RemoteFilesLocation},
    },
    remote_files_dir_pickers::selectors as remote_files_dir_pickers_selectors,
    store,
};

use super::state::{RepoCreateForm, RepoCreateState};

pub fn select_repo_create_form<'a>(state: &'a store::State) -> Option<&'a RepoCreateForm> {
    match &state.repo_create {
        Some(RepoCreateState::Form(repo_create_form)) => Some(repo_create_form),
        _ => None,
    }
}

pub fn select_location<'a>(state: &'a store::State) -> Option<&'a RemoteFilesLocation> {
    select_repo_create_form(state).and_then(|form| form.location.as_ref())
}

pub fn select_primary_mount_location(state: &store::State) -> Option<RemoteFilesLocation> {
    select_repo_create_form(state).and_then(|form| {
        form.primary_mount_id
            .as_ref()
            .map(|mount_id| RemoteFilesLocation {
                mount_id: mount_id.to_owned(),
                path: String::from("/"),
            })
    })
}

pub fn select_location_dir_picker_id(state: &store::State) -> Option<u32> {
    select_repo_create_form(state).and_then(|form| form.location_dir_picker_id)
}

pub fn select_location_dir_picker_selected_file<'a>(
    state: &'a store::State,
) -> Option<&'a RemoteFile> {
    select_location_dir_picker_id(state).and_then(|picker_id| {
        remote_files_dir_pickers_selectors::select_selected_file(state, picker_id)
    })
}

pub fn select_location_dir_picker_can_select(state: &store::State) -> bool {
    select_location_dir_picker_selected_file(state)
        .filter(|file| file.path != "/")
        .is_some()
}

pub fn select_location_dir_picker_can_show_create_dir(state: &store::State) -> bool {
    select_location_dir_picker_id(state)
        .map(|picker_id| {
            remote_files_dir_pickers_selectors::select_can_show_create_dir(state, picker_id)
        })
        .unwrap_or(false)
}

pub fn select_location_dir_picker_check_create_dir(
    state: &store::State,
    name: &str,
) -> Result<(), RemoteError> {
    let picker_id =
        select_location_dir_picker_id(state).ok_or_else(RemoteFilesErrors::not_found)?;

    remote_files_dir_pickers_selectors::select_check_create_dir(state, picker_id, name)
}

pub fn is_password_valid(password: &str) -> bool {
    password.len() >= 8
}

pub fn select_can_create(state: &store::State) -> bool {
    match &state.repo_create {
        Some(RepoCreateState::Form(RepoCreateForm {
            location, password, ..
        })) => location.is_some() && is_password_valid(password),
        _ => false,
    }
}
