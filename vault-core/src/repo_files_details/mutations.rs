use std::{collections::HashMap, sync::Arc};

use crate::{
    cipher::Cipher,
    common::state::Status,
    eventstream::mutations::{add_mount_subscriber, remove_mount_subscriber},
    remote_files::errors::RemoteFilesErrors,
    repo_files::{
        errors::{DeleteFileError, LoadFilesError},
        selectors as repo_files_selectors,
        state::{RepoFile, RepoFilesUploadResult},
    },
    repo_files_read::errors::GetFilesReaderError,
    repos, store,
    transfers::errors::TransferError,
    types::{EncryptedName, EncryptedPath, RepoId},
    utils::repo_encrypted_path_utils,
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
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    repo_id: RepoId,
    path: &EncryptedPath,
    is_editing: bool,
    details_id: u32,
    cipher: Option<&Cipher>,
) -> Result<RepoFilesDetailsLocation, LoadFilesError> {
    let path = repo_encrypted_path_utils::normalize_path(&path)
        .map_err(|_| LoadFilesError::RemoteError(RemoteFilesErrors::invalid_path()))?;

    let name = repo_encrypted_path_utils::path_to_name(&path)
        .ok_or_else(|| LoadFilesError::RemoteError(RemoteFilesErrors::invalid_path()))?;

    let decrypted_name = cipher.map(|cipher| cipher.decrypt_filename(&name));

    let eventstream_mount_subscription = repos::selectors::select_repo(state, &repo_id)
        .ok()
        .map(|repo| (repo.mount_id.clone(), repo.path.clone()))
        .map(|(mount_id, path)| {
            add_mount_subscriber(
                state,
                notify,
                mutation_state,
                mount_id,
                path,
                selectors::get_eventstream_mount_subscriber(details_id),
            )
        });

    Ok(RepoFilesDetailsLocation {
        repo_id,
        path,
        name,
        decrypted_name,
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
    })
}

fn create_status(
    state: &store::State,
    location: Result<&RepoFilesDetailsLocation, &LoadFilesError>,
) -> Status<LoadFilesError> {
    match location {
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
    }
}

pub fn create(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    options: RepoFilesDetailsOptions,
    repo_id: RepoId,
    path: &EncryptedPath,
    is_editing: bool,
    repo_files_subscription_id: u32,
    cipher: Option<&Cipher>,
) -> u32 {
    notify(store::Event::RepoFilesDetails);

    let details_id = state.repo_files_details.next_id.next();

    let location = create_location(
        state,
        notify,
        mutation_state,
        repo_id,
        path,
        is_editing,
        details_id,
        cipher,
    );

    let status = create_status(state, location.as_ref());

    let details = RepoFilesDetails {
        id: details_id,
        options,
        location: location.ok(),
        status,
        repo_files_subscription_id,
        repo_status: Status::Initial,
        is_locked: false,
    };

    state.repo_files_details.details.insert(details_id, details);

    update_details(state, notify, details_id, cipher);

    details_id
}

pub fn destroy(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    details_id: u32,
) -> (Option<u32>, Option<u32>) {
    notify(store::Event::RepoFilesDetails);

    match state.repo_files_details.details.remove(&details_id) {
        Some(details) => {
            let repo_files_subscription_id = details.repo_files_subscription_id;

            let transfer_id = details
                .location
                .as_ref()
                .and_then(|location| location.content.transfer_id);

            if let Some(mount_subscription) = details
                .location
                .and_then(|location| location.eventstream_mount_subscription)
            {
                remove_mount_subscriber(state, notify, mutation_state, mount_subscription);
            }

            (Some(repo_files_subscription_id), transfer_id)
        }
        None => (None, None),
    }
}

pub fn loading(state: &mut store::State, notify: &store::Notify, details_id: u32) {
    let details = match state.repo_files_details.details.get_mut(&details_id) {
        Some(details) => details,
        _ => return,
    };

    let new_status = Status::Loading {
        loaded: details.status.loaded(),
    };

    if details.status != new_status {
        notify(store::Event::RepoFilesDetails);

        details.status = new_status;
    }
}

pub fn loaded(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    repo_id: &RepoId,
    path: &EncryptedPath,
    error: Option<&LoadFilesError>,
) {
    let details = match state.repo_files_details.details.get_mut(&details_id) {
        Some(details) => details,
        _ => return,
    };

    if details
        .location
        .as_ref()
        .filter(|loc| &loc.repo_id == repo_id && &loc.path == path)
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

pub fn update_details(
    state: &mut store::State,
    notify: &store::Notify,
    details_id: u32,
    cipher: Option<&Cipher>,
) {
    let details = match state.repo_files_details.details.get(&details_id) {
        Some(details) => details,
        _ => return,
    };

    let (repo_status, is_locked) = match &details.location {
        Some(loc) => {
            let (repo, status) = repos::selectors::select_repo_status(state, &loc.repo_id);

            (
                status,
                repo.map(|repo| repo.state.is_locked()).unwrap_or(false),
            )
        }
        None => (Status::Initial, false),
    };

    let details = match state.repo_files_details.details.get_mut(&details_id) {
        Some(details) => details,
        _ => return,
    };

    let mut dirty = false;

    if let Some(location) = &mut details.location {
        let decrypted_name_is_some = location.decrypted_name.is_some();

        match (decrypted_name_is_some, cipher) {
            (false, Some(cipher)) => {
                location.decrypted_name = Some(cipher.decrypt_filename(&location.name));

                dirty = true;
            }
            (true, None) => {
                location.decrypted_name = None;

                dirty = true;
            }
            _ => {}
        }
    }

    if details.repo_status != repo_status {
        details.repo_status = repo_status;

        dirty = true;
    }

    if details.is_locked != is_locked {
        details.is_locked = is_locked;

        dirty = true;
    }

    if dirty {
        notify(store::Event::RepoFilesDetails);
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
    repo_id: &RepoId,
    path: &EncryptedPath,
    res: Result<Option<RepoFilesDetailsContentData>, TransferError>,
) {
    let location = match selectors::select_details_location_mut(state, details_id) {
        Some(location) => location,
        _ => return,
    };

    if &location.repo_id != repo_id || &location.path != path {
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
    repo_id: &RepoId,
    path: &EncryptedPath,
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
) -> Result<
    (
        RepoId,
        EncryptedPath,
        EncryptedName,
        RepoFilesDetailsContentData,
        u32,
        bool,
    ),
    SaveError,
> {
    if !selectors::select_is_dirty(state, details_id) {
        return Err(SaveError::NotDirty);
    }

    let file = selectors::select_file(state, details_id);

    let remote_file = file.and_then(|file| repo_files_selectors::select_remote_file(state, file));

    let content = selectors::select_content(state, details_id).ok_or(SaveError::InvalidState)?;

    let data = content.data.clone().ok_or(SaveError::InvalidState)?;

    let location = match selectors::select_details_location(state, details_id) {
        Some(location) => location,
        None => return Err(SaveError::InvalidState),
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
        None => return Err(SaveError::InvalidState),
    };

    location.save_status = Status::Loading {
        loaded: location.save_status.loaded(),
    };

    notify(store::Event::RepoFilesDetails);

    Ok((
        location.repo_id.clone(),
        location.path.clone(),
        location.name.clone(),
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
    res: Result<(EncryptedPath, RepoFilesUploadResult, bool), SaveError>,
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

pub fn handle_mutation(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    ciphers: &HashMap<RepoId, Arc<Cipher>>,
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
                    if let Some(new_name) = repo_encrypted_path_utils::path_to_name(new_path) {
                        location.path = new_path.to_owned();
                        location.name = new_name;
                        location.decrypted_name = None;

                        notify(store::Event::RepoFilesDetails);
                    }
                }
            }
        }
    }

    for (details_id, cipher) in state
        .repo_files_details
        .details
        .values()
        .map(|details| {
            (
                details.id,
                details
                    .location
                    .as_ref()
                    .and_then(|loc| ciphers.get(&loc.repo_id).cloned()),
            )
        })
        .collect::<Vec<_>>()
    {
        update_details(state, notify, details_id, cipher.as_deref())
    }
}
