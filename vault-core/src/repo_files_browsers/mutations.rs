use std::collections::HashSet;

use crate::{
    common::state::Status,
    repo_files::{
        errors::LoadFilesError, selectors as repo_files_selectors, state::RepoFilesSortField,
    },
    selection::{mutations as selection_mutations, state::Selection},
    sort::state::SortDirection,
    store,
    utils::path_utils,
};

use super::{
    selectors,
    state::{RepoFilesBrowser, RepoFilesBrowserLocation, RepoFilesBrowserOptions},
};

fn get_initial_status(
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
    options: RepoFilesBrowserOptions,
    location: Result<RepoFilesBrowserLocation, LoadFilesError>,
) -> u32 {
    notify(store::Event::RepoFilesBrowsers);

    let browser_id = state.repo_files_browsers.next_id.next();

    let status = get_initial_status(state, location.as_ref());

    let browser = RepoFilesBrowser {
        options,
        location: location.ok(),
        status,
        file_ids: Vec::new(),
        selection: Selection::default(),
        sort: Default::default(),
    };

    state
        .repo_files_browsers
        .browsers
        .insert(browser_id, browser);

    update_files(state, notify, browser_id);

    browser_id
}

pub fn destroy(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    notify(store::Event::RepoFilesBrowsers);

    state.repo_files_browsers.browsers.remove(&browser_id);
}

pub fn set_location(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    location: Result<RepoFilesBrowserLocation, LoadFilesError>,
) {
    let status = get_initial_status(state, location.as_ref());

    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    browser.location = location.ok();
    browser.status = status;
    browser.selection = Selection::default();

    update_files(state, notify, browser_id);
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
    repo_id: &str,
    path: &str,
    error: Option<&LoadFilesError>,
) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    if browser
        .location
        .as_ref()
        .filter(|loc| loc.repo_id == repo_id && loc.path == path)
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

    update_files(state, notify, browser_id);
}

pub fn update_files(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.repo_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let file_ids: Vec<String> = browser
        .location
        .as_ref()
        .map(|loc| {
            let file_ids: Vec<String> = selectors::select_file_ids(state, &loc.repo_id, &loc.path)
                .map(str::to_string)
                .collect();

            repo_files_selectors::select_sorted_files(state, &file_ids, &browser.sort)
        })
        .unwrap_or(Default::default());

    let file_ids_set: HashSet<String> = file_ids.iter().cloned().collect();

    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let mut dirty = false;

    if browser.file_ids != file_ids {
        browser.file_ids = file_ids;
        dirty = true;
    }

    let select_file_id = if let Some(name) = browser.options.select_name.clone() {
        let file_id = browser
            .location
            .as_ref()
            .map(|loc| {
                repo_files_selectors::get_file_id(
                    &loc.repo_id,
                    &path_utils::join_path_name(&loc.path, &name),
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

    if selection_mutations::update_selection(&mut browser.selection, file_ids_set) {
        dirty = true;
    }

    if let Some(file_id) = select_file_id {
        select_file(state, notify, browser_id, &file_id, false, false, true);

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
    file_id: &str,
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
                .map(str::to_string)
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
                .map(str::to_string)
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
    selection: Vec<String>,
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

    update_files(state, notify, browser_id);
}

pub fn handle_repo_files_mutation(state: &mut store::State, notify: &store::Notify) {
    for browser_id in state
        .repo_files_browsers
        .browsers
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        update_files(state, notify, browser_id)
    }
}
