use crate::common::state::Status;
use crate::repo_files::errors::LoadFilesError;
use crate::repo_files::selectors as repo_files_selectors;
use crate::store;

use super::state::{RepoFilesDetails, RepoFilesDetailsLocation};

pub fn create(
    state: &mut store::State,
    location: Result<RepoFilesDetailsLocation, LoadFilesError>,
) -> u32 {
    let details_id = state.repo_files_details.next_id;

    state.repo_files_details.next_id += 1;

    let status = match &location {
        Ok(location) => {
            if repo_files_selectors::select_file(
                state,
                &repo_files_selectors::get_file_id(&location.repo_id, &location.path),
            )
            .is_some()
            {
                Status::Reloading
            } else {
                Status::Loading
            }
        }
        Err(err) => Status::Error { error: err.clone() },
    };

    let details = RepoFilesDetails {
        location: location.ok(),
        status,
    };

    state.repo_files_details.details.insert(details_id, details);

    details_id
}

pub fn destroy(state: &mut store::State, details_id: u32) {
    state.repo_files_details.details.remove(&details_id);
}

pub fn loaded(
    state: &mut store::State,
    details_id: u32,
    repo_id: &str,
    path: &str,
    error: Option<&LoadFilesError>,
) {
    let mut details = match state.repo_files_details.details.get_mut(&details_id) {
        Some(details) => details,
        _ => return,
    };

    if details
        .location
        .as_ref()
        .filter(|loc| loc.repo_id == repo_id && loc.path == path)
        .is_some()
    {
        details.status = match error {
            Some(error) => Status::Error {
                error: error.clone(),
            },
            None => Status::Loaded,
        };
    }
}
