use crate::{
    common::state::Status,
    repo_files::{
        errors::{LoadFilesError, RepoFilesErrors},
        selectors as repo_files_selectors,
        state::RepoFile,
    },
    repos::{
        selectors as repo_selectors,
        state::{Repo, RepoState},
    },
    store,
    utils::path_utils,
};

use super::state::{RepoFilesDetails, RepoFilesDetailsInfo, RepoFilesDetailsLocation};

pub fn select_details<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetails> {
    state.repo_files_details.details.get(&details_id)
}

pub fn select_details_location<'a>(
    state: &'a store::State,
    details_id: u32,
) -> Option<&'a RepoFilesDetailsLocation> {
    select_details(state, details_id).and_then(|details| details.location.as_ref())
}

pub fn select_repo_id<'a>(state: &'a store::State, details_id: u32) -> Option<&'a str> {
    select_details_location(state, details_id).map(|loc| loc.repo_id.as_str())
}

pub fn select_repo_id_path_owned(
    state: &store::State,
    details_id: u32,
) -> Option<(String, String)> {
    select_details_location(state, details_id).map(|loc| (loc.repo_id.clone(), loc.path.clone()))
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

pub fn select_info<'a>(state: &'a store::State, details_id: u32) -> Option<RepoFilesDetailsInfo> {
    select_details(state, details_id).map(|details| {
        let file_id = details
            .location
            .as_ref()
            .map(|loc| repo_files_selectors::get_file_id(&loc.repo_id, &loc.path));
        let file = file_id.and_then(|file_id| repo_files_selectors::select_file(state, &file_id));
        let can_download = true;
        let can_copy = true;
        let can_move = true;
        let can_delete = true;

        RepoFilesDetailsInfo {
            repo_id: details.location.as_ref().map(|loc| loc.repo_id.as_str()),
            parent_path: details
                .location
                .as_ref()
                .and_then(|loc| path_utils::parent_path(&loc.path)),
            path: details.location.as_ref().map(|loc| loc.path.as_str()),
            status: match &details.status {
                Status::Loaded => {
                    if file.is_some() {
                        Status::Loaded
                    } else {
                        Status::Error {
                            error: LoadFilesError::RemoteError(RepoFilesErrors::not_found()),
                        }
                    }
                }
                _ => details.status.clone(),
            },
            file,
            content_status: details
                .location
                .as_ref()
                .map(|location| location.content.status.clone())
                .unwrap_or(Status::Initial),
            can_download,
            can_copy,
            can_move,
            can_delete,
        }
    })
}

pub fn select_file_id(state: &store::State, details_id: u32) -> Option<String> {
    select_details_location(state, details_id)
        .map(|loc| repo_files_selectors::get_file_id(&loc.repo_id, &loc.path))
}

pub fn select_file<'a>(state: &'a store::State, details_id: u32) -> Option<&'a RepoFile> {
    select_file_id(state, details_id)
        .and_then(|file_id| repo_files_selectors::select_file(state, &file_id))
}

pub fn select_content_bytes<'a>(state: &'a store::State, details_id: u32) -> (Option<&[u8]>, u32) {
    select_details(state, details_id)
        .and_then(|details| details.location.as_ref())
        .map(|location| (location.content.bytes.as_deref(), location.content.version))
        .unwrap_or((None, 0))
}
