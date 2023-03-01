use std::collections::HashSet;

use crate::common::state::Status;
use crate::repo_files::errors::LoadFilesError;
use crate::repo_files::selectors as repo_files_selectors;
use crate::selection::mutations as selection_mutations;
use crate::selection::state::{Selection, SelectionSummary};
use crate::store;

use super::selectors;
use super::state::{RepoFilesBrowser, RepoFilesBrowserLocation};

fn get_initial_status(
    state: &store::State,
    location: Result<&RepoFilesBrowserLocation, &LoadFilesError>,
) -> Status<LoadFilesError> {
    match location {
        Ok(location) => {
            if repo_files_selectors::select_is_root_loaded(state, &location.repo_id, &location.path)
            {
                Status::Reloading
            } else {
                Status::Loading
            }
        }
        Err(err) => Status::Error { error: err.clone() },
    }
}

pub fn create(
    state: &mut store::State,
    location: Result<RepoFilesBrowserLocation, LoadFilesError>,
    repo_files_subscription_id: u32,
) -> u32 {
    let browser_id = state.repo_files_browsers.next_id;

    state.repo_files_browsers.next_id += 1;

    let status = get_initial_status(state, location.as_ref());

    let browser = RepoFilesBrowser {
        location: location.ok(),
        status,
        selection: Selection::default(),
        repo_files_subscription_id,
    };

    state
        .repo_files_browsers
        .browsers
        .insert(browser_id, browser);

    update_files(state, browser_id);

    browser_id
}

pub fn destroy(state: &mut store::State, browser_id: u32) -> Option<u32> {
    let repo_files_subscription_id = state
        .repo_files_browsers
        .browsers
        .get(&browser_id)
        .map(|loc| loc.repo_files_subscription_id);

    state.repo_files_browsers.browsers.remove(&browser_id);

    repo_files_subscription_id
}

pub fn set_location(
    state: &mut store::State,
    browser_id: u32,
    location: Result<RepoFilesBrowserLocation, LoadFilesError>,
) {
    let status = get_initial_status(state, location.as_ref());

    let mut browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    browser.location = location.ok();
    browser.status = status;
    browser.selection = Selection::default();
}

pub fn loaded(
    state: &mut store::State,
    browser_id: u32,
    repo_id: &str,
    path: &str,
    error: Option<&LoadFilesError>,
) {
    let mut browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    if browser
        .location
        .as_ref()
        .filter(|loc| loc.repo_id == repo_id && loc.path == path)
        .is_some()
    {
        match error {
            Some(error) => {
                browser.status = Status::Error {
                    error: error.clone(),
                }
            }
            None => browser.status = Status::Loaded,
        }
    }
}

pub fn update_files(state: &mut store::State, browser_id: u32) -> bool {
    let browser = match state.repo_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return false,
    };

    let file_ids_set: HashSet<String> = browser
        .location
        .as_ref()
        .map(|loc| {
            selectors::select_file_ids(state, &loc.repo_id, &loc.path)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or(HashSet::new());

    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return false,
    };

    selection_mutations::update_selection(&mut browser.selection, file_ids_set)
}

pub fn select_file(
    state: &mut store::State,
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

    selection_mutations::select_item(&mut browser.selection, items, file_id, extend, range, force)
}

pub fn toggle_select_all(state: &mut store::State, browser_id: u32) {
    let selection_summary = selectors::select_selection_summary(state, browser_id);

    match selection_summary {
        SelectionSummary::All => {
            let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
                Some(browser) => browser,
                _ => return,
            };

            selection_mutations::clear_selection(&mut browser.selection);
        }
        _ => {
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

            selection_mutations::set_selection(&mut browser.selection, items);
        }
    }
}

pub fn clear_selection(state: &mut store::State, browser_id: u32) {
    let browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    selection_mutations::clear_selection(&mut browser.selection);
}
