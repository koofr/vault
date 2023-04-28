use std::sync::Arc;

use crate::repo_files::errors::LoadFilesError;
use crate::repo_files::selectors as repo_files_selectors;
use crate::repo_files_read::errors::GetFilesReaderError;
use crate::store;
use crate::{common::state::Status, eventstream::service::MountSubscription};

use super::state::{RepoFilesDetails, RepoFilesDetailsContent, RepoFilesDetailsLocation};

pub fn create_location(
    repo_id: String,
    path: String,
    eventstream_mount_subscription: Option<Arc<MountSubscription>>,
) -> RepoFilesDetailsLocation {
    RepoFilesDetailsLocation {
        repo_id,
        path,
        eventstream_mount_subscription,
        content: RepoFilesDetailsContent {
            status: Status::Initial,
            bytes: None,
            version: 0,
        },
    }
}

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

pub fn content_loading(state: &mut store::State, details_id: u32) {
    let mut location = match state
        .repo_files_details
        .details
        .get_mut(&details_id)
        .and_then(|details| details.location.as_mut())
    {
        Some(location) => location,
        _ => return,
    };

    location.content.status = Status::Loading;
}

pub fn content_loaded(
    state: &mut store::State,
    details_id: u32,
    repo_id: String,
    path: String,
    res: Result<Vec<u8>, GetFilesReaderError>,
) {
    let mut location = match state
        .repo_files_details
        .details
        .get_mut(&details_id)
        .and_then(|details| details.location.as_mut())
    {
        Some(location) => location,
        _ => return,
    };

    if location.repo_id != repo_id || location.path != path {
        return;
    }

    match res {
        Ok(bytes) => {
            location.content.status = Status::Loaded;
            location.content.bytes = Some(bytes);
            location.content.version += 1;
        }
        Err(err) => {
            location.content.status = Status::Error { error: err };
        }
    }
}
