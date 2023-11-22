use crate::{
    common::state::Status,
    remote::ApiErrorCode,
    remote_files::state::RemoteFile,
    repo_files::{
        errors::{LoadFilesError, RepoFilesErrors},
        selectors as repo_files_selectors,
        state::RepoFile,
    },
    repo_files_read::errors::GetFilesReaderError,
    repos::{
        selectors as repo_selectors,
        state::{Repo, RepoState},
    },
    store,
    transfers::errors::TransferError,
    types::{DecryptedName, DecryptedPath, EncryptedPath, RepoFileId, RepoId},
    user_error::UserError,
    utils::repo_path_utils,
};

use super::{
    errors::SaveError,
    state::{
        RepoFilesDetails, RepoFilesDetailsContent, RepoFilesDetailsContentData,
        RepoFilesDetailsContentLoading, RepoFilesDetailsInfo, RepoFilesDetailsLocation,
    },
};

pub fn get_eventstream_mount_subscriber(details_id: u32) -> String {
    format!("RepoFilesDetails:{}", details_id)
}

pub fn select_details<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetails> {
    state.repo_files_details.details.get(&details_id)
}

pub fn select_details_mut<'a>(
    state: &'a mut store::State,
    details_id: u32,
) -> Option<&'a mut RepoFilesDetails> {
    state.repo_files_details.details.get_mut(&details_id)
}

pub fn select_details_location<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetailsLocation> {
    select_details(state, details_id).and_then(|details| details.location.as_ref())
}

pub fn select_details_location_mut<'a>(
    state: &'a mut store::State,
    details_id: u32,
) -> Option<&'a mut RepoFilesDetailsLocation> {
    select_details_mut(state, details_id).and_then(|details| details.location.as_mut())
}

pub fn select_repo_id<'a>(state: &'a store::State, details_id: u32) -> Option<&'a RepoId> {
    select_details_location(state, details_id).map(|loc| &loc.repo_id)
}

pub fn select_repo_id_path_owned(
    state: &store::State,
    details_id: u32,
) -> Option<(RepoId, DecryptedPath, EncryptedPath)> {
    select_details_location(state, details_id).map(|loc| {
        (
            loc.repo_id.clone(),
            loc.path.clone(),
            loc.encrypted_path.clone(),
        )
    })
}

pub fn select_repo<'a>(state: &'a store::State, details_id: u32) -> Option<&'a Repo> {
    select_details(state, details_id)
        .and_then(|details| details.location.as_ref())
        .and_then(|loc| repo_selectors::select_repo(state, &loc.repo_id).ok())
}

pub fn select_repo_state<'a>(state: &'a store::State, details_id: u32) -> Option<&'a RepoState> {
    select_repo(state, details_id).map(|repo| &repo.state)
}

pub fn select_is_unlocked<'a>(state: &'a store::State, details_id: u32) -> bool {
    select_repo_state(state, details_id)
        .map(|repo_state| repo_state.is_unlocked())
        .unwrap_or(false)
}

pub fn select_is_status_any_loaded(state: &store::State, details_id: u32) -> bool {
    select_details(state, details_id)
        .map(|details| details.status.loaded())
        .unwrap_or(false)
}

pub fn get_status(status: &Status<LoadFilesError>, file_exists: bool) -> Status<LoadFilesError> {
    match status {
        Status::Loaded => {
            if file_exists {
                Status::Loaded
            } else {
                Status::Error {
                    error: LoadFilesError::RemoteError(RepoFilesErrors::not_found()),
                    loaded: true,
                }
            }
        }
        _ => status.to_owned(),
    }
}

pub fn get_is_content_conflict(
    is_dirty: bool,
    data: Option<&RepoFilesDetailsContentData>,
    remote_file: Option<&RemoteFile>,
) -> bool {
    is_dirty
        && match (data, remote_file) {
            (Some(data), Some(remote_file)) => !content_data_matches_remote_file(data, remote_file),
            _ => false,
        }
}

pub fn get_conflict_error(is_conflict: bool) -> Option<String> {
    if is_conflict {
        Some(String::from(
            "File was changed by someone else since your last save. Automatic saving is disabled.",
        ))
    } else {
        None
    }
}

pub fn get_is_save_conflict(status: &Status<SaveError>) -> bool {
    match status {
        Status::Error { error, .. } => match error {
            SaveError::RemoteError(error) => error.is_api_error_code(ApiErrorCode::Conflict),
            _ => false,
        },
        _ => false,
    }
}

pub fn get_is_conflict(
    is_dirty: bool,
    data: Option<&RepoFilesDetailsContentData>,
    remote_file: Option<&RemoteFile>,
    save_status: &Status<SaveError>,
) -> bool {
    !matches!(save_status, Status::Loading { loaded: false })
        && (get_is_content_conflict(is_dirty, data, remote_file)
            || get_is_save_conflict(save_status))
}

pub fn get_save_error(status: &Status<SaveError>) -> Option<String> {
    if get_is_save_conflict(status) {
        get_conflict_error(true)
    } else {
        match status {
            Status::Error { error, .. } => Some(error.user_error()),
            _ => None,
        }
    }
}

pub fn get_load_error(status: &Status<LoadFilesError>) -> Option<String> {
    match status {
        Status::Error { error, .. } => match error {
            LoadFilesError::RemoteError(error) if error.is_api_error_code(ApiErrorCode::NotFound) => Some(String::from("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")),
            _ => Some(error.user_error()),
        },
        _ => None,
    }
}

pub fn get_content_error(status: &Status<TransferError>) -> Option<String> {
    match status {
        Status::Error { error, .. } => match error {
            TransferError::RemoteError(error) if error.is_api_error_code(ApiErrorCode::NotFound) => Some(String::from("This file is no longer accessible. Probably it was deleted or you no longer have access to it.")),
            _ => Some(error.user_error()),
        },
        _ => None,
    }
}

pub fn content_data_matches_remote_file(
    data: &RepoFilesDetailsContentData,
    remote_file: &RemoteFile,
) -> bool {
    data.remote_size == remote_file.size
        && data.remote_modified == remote_file.modified
        && data.remote_hash == remote_file.hash
}

pub fn content_loading_matches_remote_file(
    loading: &RepoFilesDetailsContentLoading,
    remote_file: &RemoteFile,
) -> bool {
    loading.remote_size == remote_file.size
        && loading.remote_modified == remote_file.modified
        && loading.remote_hash == remote_file.hash
}

pub fn select_info<'a>(state: &'a store::State, details_id: u32) -> Option<RepoFilesDetailsInfo> {
    select_details(state, details_id).map(|details| {
        let location = details.location.as_ref();
        let repo_id = location.map(|loc| &loc.repo_id);
        let parent_path = location.and_then(|loc| repo_path_utils::parent_path(&loc.path));
        let path = location.map(|loc| &loc.path);
        let file_id = location
            .map(|loc| repo_files_selectors::get_file_id(&loc.repo_id, &loc.encrypted_path));
        let file = file_id.and_then(|file_id| repo_files_selectors::select_file(state, &file_id));
        let (file_name, file_ext, file_category) = {
            file.map(|file| {
                (
                    repo_files_selectors::select_file_name(state, file)
                        .ok()
                        .map(ToOwned::to_owned),
                    file.ext.clone(),
                    Some(file.category.clone()),
                )
            })
            .unwrap_or_else(move || {
                match path.and_then(repo_path_utils::path_to_name) {
                    Some(name) => {
                        let (ext, _, category) =
                            repo_files_selectors::get_file_ext_content_type_category(
                                &name.to_lowercase().0,
                            );

                        (Some(name), ext, Some(category))
                    }
                    None => (None, None, None),
                }
            })
        };
        let file_modified = file.and_then(|file| file.modified);
        let file_exists = file.is_some();
        let remote_file =
            file.and_then(|file| repo_files_selectors::select_remote_file(state, file));
        let status = get_status(&details.status, file.is_some());
        let content_status = location
            .map(|location| location.content.status.clone())
            .unwrap_or(Status::Initial);
        let transfer_id = location.and_then(|location| location.content.transfer_id);
        let save_status = location
            .map(|location| location.save_status.clone())
            .unwrap_or(Status::Initial);
        let content_data = location.and_then(|loc| loc.content.data.as_ref());
        let is_dirty = location.map(|loc| loc.is_dirty).unwrap_or(false);
        let should_destroy = location.map(|loc| loc.should_destroy).unwrap_or(false);
        let is_conflict = get_is_conflict(is_dirty, content_data, remote_file, &save_status);
        let error = get_save_error(&save_status)
            .or_else(|| get_load_error(&status))
            .or_else(|| get_content_error(&content_status))
            .or_else(|| get_conflict_error(is_conflict));
        let is_editing = location.map(|loc| loc.is_editing).unwrap_or(false);
        let can_save = is_editing && is_dirty && !matches!(save_status, Status::Loading { .. });
        let can_download = true;
        let can_copy = true;
        let can_move = true;
        let can_delete = true;

        RepoFilesDetailsInfo {
            repo_id,
            parent_path,
            path,
            status,
            file_name,
            file_ext,
            file_category,
            file_modified,
            file_exists,
            content_status,
            transfer_id,
            save_status,
            error,
            is_editing,
            is_dirty,
            should_destroy,
            can_save,
            can_download,
            can_copy,
            can_move,
            can_delete,
        }
    })
}

pub fn select_file_id(state: &store::State, details_id: u32) -> Option<RepoFileId> {
    select_details_location(state, details_id)
        .map(|loc| repo_files_selectors::get_file_id(&loc.repo_id, &loc.encrypted_path))
}

pub fn select_file<'a>(state: &'a store::State, details_id: u32) -> Option<&'a RepoFile> {
    select_file_id(state, details_id)
        .and_then(|file_id| repo_files_selectors::select_file(state, &file_id))
}

pub fn select_file_name<'a>(state: &'a store::State, details_id: u32) -> Option<DecryptedName> {
    select_file(state, details_id)
        .and_then(|file| {
            repo_files_selectors::select_file_name(state, file)
                .ok()
                .map(ToOwned::to_owned)
        })
        .or_else(|| {
            select_details_location(state, details_id)
                .map(|loc| &loc.path)
                .and_then(repo_path_utils::path_to_name)
        })
}

pub fn select_remote_file<'a>(state: &'a store::State, details_id: u32) -> Option<&'a RemoteFile> {
    let file = select_file(state, details_id);

    file.and_then(|file| repo_files_selectors::select_remote_file(state, file))
}

pub fn select_is_editing(state: &store::State, details_id: u32) -> bool {
    select_details_location(state, details_id)
        .map(|loc| loc.is_editing)
        .unwrap_or(false)
}

pub fn select_is_dirty(state: &store::State, details_id: u32) -> bool {
    select_details_location(state, details_id)
        .map(|loc| loc.is_dirty)
        .unwrap_or(false)
}

pub fn select_is_content_loading(state: &store::State, details_id: u32) -> bool {
    select_details_location(state, details_id)
        .map(|loc| matches!(loc.content.status, Status::Loading { .. }))
        .unwrap_or(false)
}

pub fn select_is_content_loaded_error(state: &store::State, details_id: u32) -> bool {
    select_details_location(state, details_id)
        .map(|loc| matches!(loc.content.status, Status::Error { .. }))
        .unwrap_or(false)
}

pub fn select_is_saving(state: &store::State, details_id: u32) -> bool {
    select_details_location(state, details_id)
        .map(|loc| matches!(loc.save_status, Status::Loading { .. }))
        .unwrap_or(false)
}

pub fn select_content_bytes_version<'a>(
    state: &'a store::State,
    details_id: u32,
) -> (Option<&'a [u8]>, u32) {
    select_details(state, details_id)
        .and_then(|details| details.location.as_ref())
        .map(|location| {
            (
                location
                    .content
                    .data
                    .as_ref()
                    .map(|data| data.bytes.as_ref()),
                location.content.version,
            )
        })
        .unwrap_or((None, 0))
}

pub fn select_content<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetailsContent> {
    select_details(state, details_id)
        .and_then(|details| details.location.as_ref())
        .map(|location| &location.content)
}

pub fn select_content_data<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetailsContentData> {
    select_content(state, details_id).and_then(|content| content.data.as_ref())
}

pub fn select_content_loading<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetailsContentLoading> {
    select_content(state, details_id).and_then(|content| content.loading.as_ref())
}

pub fn select_is_content_stale<'a>(state: &'a store::State, details_id: u32) -> bool {
    let remote_file = select_remote_file(state, details_id);
    let content_data = select_content_data(state, details_id);
    let content_loading = select_content_loading(state, details_id);

    match (remote_file, content_data, content_loading) {
        (Some(remote_file), Some(data), Some(loading)) => {
            !content_data_matches_remote_file(data, remote_file)
                || !content_loading_matches_remote_file(loading, remote_file)
        }
        (Some(remote_file), Some(data), None) => {
            !content_data_matches_remote_file(data, remote_file)
        }
        (Some(remote_file), None, Some(loading)) => {
            !content_loading_matches_remote_file(loading, remote_file)
        }
        (Some(_), None, None) => true,
        (None, _, _) => false,
    }
}

pub fn select_is_not_deleting_or_deleted(state: &store::State, details_id: u32) -> bool {
    select_details_location(state, details_id)
        .map(|loc| {
            !matches!(loc.delete_status, Status::Loading { .. })
                && !matches!(loc.delete_status, Status::Loaded)
        })
        .unwrap_or(false)
}

pub fn select_was_removed(
    state: &store::State,
    mutation_state: &store::MutationState,
    details_id: u32,
) -> bool {
    !mutation_state.repo_files.removed_files.is_empty()
        && select_details_location(state, details_id)
            .map(|loc| {
                mutation_state
                    .repo_files
                    .removed_files
                    .contains(&(loc.repo_id.clone(), loc.path.clone()))
            })
            .unwrap_or(false)
}

pub fn select_should_reload_content(
    state: &store::State,
    mutation_state: &store::MutationState,
    details_id: u32,
) -> bool {
    select_is_status_any_loaded(state, details_id)
        && select_is_content_stale(state, details_id)
        && !select_is_dirty(state, details_id)
        && !select_is_saving(state, details_id)
        && !select_is_content_loading(state, details_id)
        && !select_is_content_loaded_error(state, details_id)
        && !select_was_removed(state, mutation_state, details_id)
}

pub fn select_should_wait_for_loaded(state: &store::State, details_id: u32) -> Option<()> {
    match select_details(state, details_id) {
        Some(details) => match &details.status {
            Status::Initial | Status::Loading { .. } => None,
            Status::Loaded | Status::Error { .. } => Some(()),
        },
        None => {
            // details not found, stop waiting
            Some(())
        }
    }
}

pub fn select_file_reader_file(
    state: &store::State,
    details_id: u32,
) -> Option<Result<RepoFile, GetFilesReaderError>> {
    match select_should_wait_for_loaded(state, details_id) {
        Some(()) => Some(
            select_file(state, details_id)
                .map(|file| file.clone())
                .ok_or(GetFilesReaderError::FileNotFound),
        ),
        None => None,
    }
}
