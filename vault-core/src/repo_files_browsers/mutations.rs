use std::collections::HashSet;
use std::sync::Arc;

use crate::common::state::Status;
use crate::eventstream::service::MountSubscription;
use crate::repo_files::errors::LoadFilesError;
use crate::repo_files::selectors as repo_files_selectors;
use crate::selection::mutations as selection_mutations;
use crate::selection::state::{Selection, SelectionSummary};
use crate::store;

use super::selectors;
use super::state::RepoFilesBrowser;

pub fn create(
    state: &mut store::State,
    repo_id: &str,
    path: &str,
    eventstream_mount_subscription: Option<Arc<MountSubscription>>,
    repo_files_subscription_id: u32,
) -> u32 {
    let browser_id = state.repo_files_browsers.next_id;

    state.repo_files_browsers.next_id += 1;

    let status = if repo_files_selectors::select_is_root_loaded(state, &repo_id, &path) {
        Status::Reloading
    } else {
        Status::Loading
    };

    let browser = RepoFilesBrowser {
        repo_id: repo_id.to_owned(),
        path: path.to_owned(),
        status: status,
        selection: Selection::default(),
        eventstream_mount_subscription,
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
        .map(|x| x.repo_files_subscription_id);

    state.repo_files_browsers.browsers.remove(&browser_id);

    repo_files_subscription_id
}

pub fn set_location(
    state: &mut store::State,
    browser_id: u32,
    repo_id: &str,
    path: &str,
    eventstream_mount_subscription: Option<Arc<MountSubscription>>,
) {
    let status = if repo_files_selectors::select_is_root_loaded(state, &repo_id, &path) {
        Status::Reloading
    } else {
        Status::Loading
    };

    let mut browser = match state.repo_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    browser.repo_id = repo_id.to_owned();
    browser.path = path.to_owned();
    browser.status = status;
    browser.selection = Selection::default();
    browser.eventstream_mount_subscription = eventstream_mount_subscription;
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

    if browser.repo_id.as_str() == repo_id && browser.path.as_str() == path {
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

    let file_ids_set: HashSet<String> =
        selectors::select_file_ids(state, &browser.repo_id, &browser.path)
            .map(str::to_string)
            .collect();

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

    let items = selectors::select_file_ids(state, &browser.repo_id, &browser.path)
        .map(str::to_string)
        .collect();

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

            let items = selectors::select_file_ids(state, &browser.repo_id, &browser.path)
                .map(str::to_string)
                .collect();

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
