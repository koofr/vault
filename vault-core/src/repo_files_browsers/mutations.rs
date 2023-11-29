use std::collections::HashSet;

use crate::{
    common::state::Status,
    eventstream::mutations::{add_mount_subscriber, remove_mount_subscriber},
    remote_files::errors::RemoteFilesErrors,
    repo_files::{
        errors::LoadFilesError, selectors as repo_files_selectors, state::RepoFilesSortField,
    },
    repos,
    selection::{mutations as selection_mutations, state::Selection},
    sort::state::SortDirection,
    store,
    types::{EncryptedPath, RepoFileId, RepoId},
    utils::repo_encrypted_path_utils,
};

use super::{
    selectors,
    state::{RepoFilesBrowser, RepoFilesBrowserLocation, RepoFilesBrowserOptions},
};

fn create_location(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    repo_id: RepoId,
    path: &EncryptedPath,
    browser_id: u32,
) -> Result<RepoFilesBrowserLocation, LoadFilesError> {
    let path = repo_encrypted_path_utils::normalize_path(path)
        .map_err(|_| LoadFilesError::RemoteError(RemoteFilesErrors::invalid_path()))?;

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
                selectors::get_eventstream_mount_subscriber(browser_id),
            )
        });

    Ok(RepoFilesBrowserLocation {
        repo_id,
        path,
        eventstream_mount_subscription,
    })
}

fn create_status(
    state: &store::State,
    location: Result<&RepoFilesBrowserLocation, &LoadFilesError>,
) -> Status<LoadFilesError> {
    match location {
        Ok(location) => Status::Loading {
            loaded: repo_files_selectors::select_is_root_loaded(
                state,
                &location.repo_id,
                &location.path,
            ),
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
    options: RepoFilesBrowserOptions,
    repo_id: RepoId,
    path: &EncryptedPath,
) -> u32 {
    notify(store::Event::RepoFilesBrowsers);

    let browser_id = state.repo_files_browsers.next_id.next();

    let location = create_location(state, notify, mutation_state, repo_id, path, browser_id);

    let status = create_status(state, location.as_ref());

    let browser = RepoFilesBrowser {
        id: browser_id,
        options,
        location: location.ok(),
        status,
        breadcrumbs: None,
        file_ids: Vec::new(),
        selection: Selection::default(),
        sort: state.repo_files_browsers.last_sort.clone(),
        repo_status: Status::Initial,
        is_locked: false,
    };

    state
        .repo_files_browsers
        .browsers
        .insert(browser_id, browser);

    update_browser(state, notify, browser_id);

    browser_id
}

pub fn destroy(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    browser_id: u32,
) {
    notify(store::Event::RepoFilesBrowsers);

    if let Some(browser) = state.repo_files_browsers.browsers.remove(&browser_id) {
        if let Some(mount_subscription) = browser
            .location
            .and_then(|location| location.eventstream_mount_subscription)
        {
            remove_mount_subscriber(state, notify, mutation_state, mount_subscription);
        }
    }
}

pub fn loading(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let new_status = Status::Loading {
        loaded: browser.status.loaded(),
    };

    if browser.status != new_status {
        notify(store::Event::RepoFilesBrowsers);

        browser.status = new_status;
    }
}

pub fn loaded(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    repo_id: &RepoId,
    path: &EncryptedPath,
    error: Option<&LoadFilesError>,
) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    if browser
        .location
        .as_ref()
        .filter(|loc| &loc.repo_id == repo_id && &loc.path == path)
        .is_some()
    {
        notify(store::Event::RepoFilesBrowsers);

        match error {
            Some(error) => {
                browser.status = Status::Error {
                    error: error.clone(),
                    loaded: browser.status.loaded(),
                }
            }
            None => browser.status = Status::Loaded,
        }
    }

    update_browser(state, notify, browser_id);
}

pub fn update_browser(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.repo_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let (repo_status, is_locked, cipher) = match &browser.location {
        Some(loc) => {
            let (repo, status) = repos::selectors::select_repo_status(state, &loc.repo_id);
            let cipher = repos::selectors::select_cipher_owned(state, &loc.repo_id).ok();

            (
                status,
                repo.map(|repo| repo.state.is_locked()).unwrap_or(false),
                cipher,
            )
        }
        None => (Status::Initial, false, None),
    };

    let update_breadcrumbs = match (&browser.breadcrumbs, cipher.as_deref()) {
        (None, Some(cipher)) => Some(browser.location.as_ref().and_then(|loc| {
            Some(repo_files_selectors::select_breadcrumbs(
                state,
                &loc.repo_id,
                &loc.path,
                &cipher,
            ))
        })),
        (Some(_), None) => Some(None),
        _ => None,
    };

    let file_ids: Vec<RepoFileId> = browser
        .location
        .as_ref()
        .map(|loc| {
            let file_ids: Vec<RepoFileId> =
                selectors::select_file_ids(state, &loc.repo_id, &loc.path)
                    .map(ToOwned::to_owned)
                    .collect();

            repo_files_selectors::select_sorted_files(state, &file_ids, &browser.sort)
        })
        .unwrap_or(Default::default());

    let file_ids_set: HashSet<RepoFileId> = file_ids.iter().cloned().collect();

    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let mut dirty = false;

    if let Some(breadcrumbs) = update_breadcrumbs {
        browser.breadcrumbs = breadcrumbs;
        dirty = true;
    }

    if browser.file_ids != file_ids {
        browser.file_ids = file_ids;
        dirty = true;
    }

    let select_file_id = if let Some(name) = browser.options.select_name.clone().and_then(|name| {
        cipher
            .as_deref()
            .map(|cipher| cipher.encrypt_filename(&name))
    }) {
        let file_id = browser
            .location
            .as_ref()
            .map(|loc| {
                repo_files_selectors::get_file_id(
                    &loc.repo_id,
                    &repo_encrypted_path_utils::join_path_name(&loc.path, &name),
                )
            })
            .filter(|file_id| file_ids_set.contains(file_id));

        if matches!(&browser.status, Status::Loaded) || file_id.is_some() {
            browser.options.select_name = None;
        }

        file_id
    } else {
        None
    };

    if !is_locked {
        if selection_mutations::update_selection(&mut browser.selection, file_ids_set) {
            dirty = true;
        }
    }

    if browser.repo_status != repo_status {
        browser.repo_status = repo_status;

        dirty = true;
    }

    if browser.is_locked != is_locked {
        browser.is_locked = is_locked;

        dirty = true;
    }

    if let Some(file_id) = select_file_id {
        select_file(state, notify, browser_id, file_id, false, false, true);

        dirty = true;
    }

    if dirty {
        notify(store::Event::RepoFilesBrowsers);
    }
}

pub fn select_file(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    file_id: RepoFileId,
    extend: bool,
    range: bool,
    force: bool,
) {
    let browser = match state.repo_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let items = browser
        .location
        .as_ref()
        .map(|loc| {
            selectors::select_file_ids(state, &loc.repo_id, &loc.path)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or(Vec::new());

    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RepoFilesBrowsers);

    selection_mutations::select_item(&mut browser.selection, items, file_id, extend, range, force)
}

pub fn select_all(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.repo_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let items = browser
        .location
        .as_ref()
        .map(|loc| {
            selectors::select_file_ids(state, &loc.repo_id, &loc.path)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or(Vec::new());

    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RepoFilesBrowsers);

    selection_mutations::set_selection(&mut browser.selection, items);
}

pub fn clear_selection(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RepoFilesBrowsers);

    selection_mutations::clear_selection(&mut browser.selection);
}

pub fn set_selection(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    selection: Vec<RepoFileId>,
) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RepoFilesBrowsers);

    selection_mutations::set_selection(&mut browser.selection, selection);
}

pub fn sort_by(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    field: RepoFilesSortField,
    direction: Option<SortDirection>,
) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RepoFilesBrowsers);

    let direction = direction.unwrap_or_else(|| {
        if browser.sort.field == field {
            browser.sort.direction.clone().reverse()
        } else {
            match field {
                RepoFilesSortField::Size | RepoFilesSortField::Modified => SortDirection::Desc,
                _ => SortDirection::Asc,
            }
        }
    });

    browser.sort.field = field;
    browser.sort.direction = direction;

    state.repo_files_browsers.last_sort = browser.sort.clone();

    update_browser(state, notify, browser_id);
}

pub fn handle_mutation(state: &mut store::State, notify: &store::Notify) {
    for browser_id in state
        .repo_files_browsers
        .browsers
        .keys()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
    {
        update_browser(state, notify, browser_id)
    }
}
