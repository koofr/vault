use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    repo_files::{
        errors::{DeleteFileError, LoadFilesError},
        selectors as repo_files_selectors,
        state::{RepoFile, RepoFilesUploadResult},
    },
    repo_files_read::errors::GetFilesReaderError,
    store,
    transfers::errors::TransferError,
};

use super::{
    errors::{LoadContentError, SaveError},
    selectors,
    state::{
        RepoFilesDetails, RepoFilesDetailsContent, RepoFilesDetailsContentData,
        RepoFilesDetailsContentLoading, RepoFilesDetailsLocation, RepoFilesDetailsOptions,
        SaveInitiator,
    },
};

pub fn create_location(
    repo_id: String,
    path: String,
    eventstream_mount_subscription: Option<Arc<MountSubscription>>,
    is_editing: bool,
) -> RepoFilesDetailsLocation {
    RepoFilesDetailsLocation {
        repo_id,
        path,
        eventstream_mount_subscription,
        content: RepoFilesDetailsContent {
            status: Status::Initial,
            data: None,
            loading: None,
            version: 0,
            transfer_id: None,
        },
        is_editing,
        is_dirty: false,
        save_status: Status::Initial,
        delete_status: Status::Initial,
        should_destroy: false,
    }
}

pub fn create(
    state: &mut store::State,
    notify: &store::Notify,
    options: RepoFilesDetailsOptions,
    location: Result<RepoFilesDetailsLocation, LoadFilesError>,
    repo_files_subscription_id: u32,
) -> u32 {
    notify(store::Event::RepoFilesDetails);

    let details_id = state.repo_files_details.next_id.next();

    let status = match &location {
        Ok(location) => Status::Loading {
            loaded: repo_files_selectors::select_file(
                state,
                &repo_files_selectors::get_file_id(&location.repo_id, &location.path),
            )
            .is_some(),
        },
        Err(err) => Status::Error {
            error: err.clone(),
            loaded: false,
        },
    };

    let details = RepoFilesDetails {
        options,
        location: location.ok(),
        status,
        repo_files_subscription_id,
    };

    state.repo_files_details.details.insert(details_id, details);

    details_id
}

pub fn destroy(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
) -> (Option<u32>, Option<u32>) {
    notify(store::Event::RepoFilesDetails);

    let repo_files_subscription_id = state
        .repo_files_details
        .details
        .get(&details_id)
        .map(|details| details.repo_files_subscription_id);

    let transfer_id = state
        .repo_files_details
        .details
        .get(&details_id)
        .and_then(|details| details.location.as_ref())
        .and_then(|location| location.content.transfer_id);

    state.repo_files_details.details.remove(&details_id);

    (repo_files_subscription_id, transfer_id)
}

pub fn loaded(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    repo_id: &str,
    path: &str,
    error: Option<&LoadFilesError>,
) {
    let details = match state.repo_files_details.details.get_mut(&details_id) {
        Some(details) => details,
        _ => return,
    };

    if details
        .location
        .as_ref()
        .filter(|loc| loc.repo_id == repo_id && loc.path == path)
        .is_some()
    {
        notify(store::Event::RepoFilesDetails);

        details.status = match error {
            Some(error) => Status::Error {
                error: error.clone(),
                loaded: details.status.loaded(),
            },
            None => Status::Loaded,
        };
    }
}

pub fn content_loading(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
) -> Result<RepoFile, LoadContentError> {
    let file = selectors::select_file(state, details_id)
        .map(|file| file.clone())
        .ok_or(LoadContentError::FileNotFound)?;

    match selectors::select_details(state, details_id) {
        Some(details)
            if details
                .options
                .load_content
                .matches(file.ext.as_deref(), &file.category) => {}
        _ => return Err(LoadContentError::LoadFilterMismatch),
    };

    let loading = selectors::select_remote_file(state, details_id).map(|remote_file| {
        RepoFilesDetailsContentLoading {
            remote_size: remote_file.size,
            remote_modified: remote_file.modified,
            remote_hash: remote_file.hash.clone(),
        }
    });

    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return Err(LoadContentError::FileNotFound),
    };

    location.content.status = match location.content.status {
        Status::Initial => Status::Loading { loaded: false },
        Status::Loaded | Status::Error { .. } => Status::Loading {
            loaded: location.content.status.loaded(),
        },
        Status::Loading { .. } => return Err(LoadContentError::AlreadyLoading),
    };
    location.content.loading = loading;

    notify(store::Event::RepoFilesDetails);

    Ok(file)
}

pub fn file_reader_loading(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    file: &RepoFile,
) -> Result<(), GetFilesReaderError> {
    let loading = repo_files_selectors::select_remote_file(state, &file).map(|remote_file| {
        RepoFilesDetailsContentLoading {
            remote_size: remote_file.size,
            remote_modified: remote_file.modified,
            remote_hash: remote_file.hash.clone(),
        }
    });

    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return Err(GetFilesReaderError::FileNotFound),
    };

    notify(store::Event::RepoFilesDetails);

    location.content.status = Status::Loading {
        loaded: location.content.status.loaded(),
    };
    location.content.loading = loading;

    Ok(())
}

pub fn content_loaded(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    repo_id: &str,
    path: &str,
    res: Result<Option<RepoFilesDetailsContentData>, TransferError>,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    if location.repo_id != repo_id || location.path != path {
        return;
    }

    notify(store::Event::RepoFilesDetails);

    location.content.loading = None;

    if location.is_dirty || matches!(location.save_status, Status::Loading { .. }) {
        location.content.status = Status::Loaded;
    } else {
        match res {
            Ok(data) => {
                location.content.status = Status::Loaded;
                location.content.data = data;
                location.content.version += 1;

                notify(store::Event::RepoFilesDetailsContentData);
            }
            Err(err) => {
                location.content.status = Status::Error {
                    error: err,
                    loaded: location.content.status.loaded(),
                };
            }
        }
    }
}

pub fn content_error(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    err: TransferError,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    notify(store::Event::RepoFilesDetails);

    location.content.status = Status::Error {
        error: err,
        loaded: location.content.status.loaded(),
    };
}

pub fn content_transfer_created(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    file: &RepoFile,
    transfer_id: u32,
) -> Result<Option<u32>, GetFilesReaderError> {
    file_reader_loading(state, notify, details_id, file)?;

    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return Ok(None),
    };

    notify(store::Event::RepoFilesDetails);

    Ok(location.content.transfer_id.replace(transfer_id))
}

pub fn content_transfer_removed(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    repo_id: &str,
    path: &str,
    transfer_id: u32,
    res: Result<(), TransferError>,
) {
    content_loaded(state, notify, details_id, repo_id, path, res.map(|_| None));

    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    if let Some(current_transfer_id) = location.content.transfer_id {
        if transfer_id == current_transfer_id {
            notify(store::Event::RepoFilesDetails);

            location.content.transfer_id = None;
        }
    }
}

pub fn edit(state: &mut store::State, notify: &store::Notify, details_id: u32) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    notify(store::Event::RepoFilesDetails);

    location.is_editing = true;
}

pub fn edit_cancel(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    is_discarded: bool,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    if location.is_editing {
        location.is_editing = false;
        location.is_dirty = false;
        location.save_status = Status::Initial;

        if is_discarded {
            // this will reload the content
            location.content.status = Status::Initial;
            location.content.data = None;
            location.content.loading = None;
        }

        notify(store::Event::RepoFilesDetails);
    }
}

pub fn set_content(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    content: Vec<u8>,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    if let Some(data) = &mut location.content.data {
        if data.bytes != content {
            data.bytes = content;

            location.content.version += 1;

            notify(store::Event::RepoFilesDetailsContentData);

            if !location.is_dirty {
                location.is_dirty = true;

                notify(store::Event::RepoFilesDetails);
            }
        }
    }
}

pub fn saving(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    initiator: SaveInitiator,
) -> Result<(String, String, RepoFilesDetailsContentData, u32, bool), SaveError> {
    if !selectors::select_is_dirty(state, details_id) {
        return Err(SaveError::NotDirty);
    }

    let file = selectors::select_file(state, details_id);

    let remote_file = file.and_then(|file| repo_files_selectors::select_remote_file(state, file));

    let content = selectors::select_content(state, details_id).ok_or(SaveError::InvalidState)?;

    let data = content.data.clone().ok_or(SaveError::InvalidState)?;

    let location = match selectors::select_details_location(state, details_id) {
        Some(location) => location,
        _ => return Err(SaveError::InvalidState),
    };

    let is_deleted = file.is_none();

    if matches!(initiator, SaveInitiator::Autosave)
        && (selectors::get_is_conflict(true, Some(&data), remote_file, &location.save_status)
            || is_deleted)
    {
        return Err(SaveError::AutosaveNotPossible);
    }

    let version = content.version;

    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return Err(SaveError::InvalidState),
    };

    location.save_status = Status::Loading {
        loaded: location.save_status.loaded(),
    };

    notify(store::Event::RepoFilesDetails);

    Ok((
        location.repo_id.clone(),
        location.path.clone(),
        data,
        version,
        is_deleted,
    ))
}

pub fn saved(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    saved_version: u32,
    res: Result<(String, RepoFilesUploadResult, bool), SaveError>,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    notify(store::Event::RepoFilesDetails);

    match res {
        Ok((saved_path, result, should_destroy)) => {
            if location.path != saved_path {
                location.path = saved_path;
            }
            if let Some(data) = &mut location.content.data {
                data.remote_size = result.remote_file.size;
                data.remote_modified = result.remote_file.modified;
                data.remote_hash = result.remote_file.hash;
            }
            if location.content.version == saved_version {
                location.is_dirty = false;
            }
            location.save_status = Status::Initial;
            if should_destroy {
                location.should_destroy = true;
            }
        }
        Err(err) => {
            match err {
                SaveError::Canceled => {
                    location.save_status = Status::Initial;
                }
                SaveError::DiscardChanges { should_destroy } => {
                    location.save_status = Status::Initial;
                    if should_destroy {
                        location.should_destroy = true;
                    }
                }
                err => {
                    location.save_status = Status::Error {
                        error: err,
                        loaded: location.save_status.loaded(),
                    };
                }
            };
        }
    }
}

pub fn deleting(state: &mut store::State, notify: &store::Notify, details_id: u32) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    notify(store::Event::RepoFilesDetails);

    location.delete_status = Status::Loading {
        loaded: location.delete_status.loaded(),
    };
}

pub fn deleted(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    res: Result<(), DeleteFileError>,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    notify(store::Event::RepoFilesDetails);

    match res {
        Ok(()) => {
            location.delete_status = Status::Loaded;
            location.should_destroy = true;
        }
        Err(DeleteFileError::Canceled) => {
            location.delete_status = Status::Initial;
        }
        Err(err) => {
            location.delete_status = Status::Error {
                error: err,
                loaded: location.delete_status.loaded(),
            };
        }
    }
}

pub fn handle_repo_files_mutation(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
) {
    if !mutation_state.repo_files.moved_files.is_empty() {
        let moved_files = mutation_state
            .repo_files
            .moved_files
            .iter()
            .map(|(repo_id, old_path, new_path)| {
                (
                    (repo_id.to_owned(), old_path.to_owned()),
                    new_path.to_owned(),
                )
            })
            .collect::<HashMap<_, _>>();

        for (_, details) in state.repo_files_details.details.iter_mut() {
            if let Some(location) = &mut details.location {
                if let Some(new_path) =
                    moved_files.get(&(location.repo_id.clone(), location.path.clone()))
                {
                    location.path = new_path.to_owned();

                    notify(store::Event::RepoFilesDetails);
                }
            }
        }
    }
}
