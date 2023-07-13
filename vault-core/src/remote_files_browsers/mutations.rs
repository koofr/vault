use std::collections::HashSet;

use crate::{
    common::state::Status,
    remote,
    remote_files::state::RemoteFilesSortField,
    selection::{mutations as selection_mutations, state::Selection},
    sort::state::SortDirection,
    store,
};

use super::{
    selectors,
    state::{
        RemoteFilesBrowser, RemoteFilesBrowserItem, RemoteFilesBrowserLocation,
        RemoteFilesBrowserOptions,
    },
};

fn get_initial_status(
    state: &store::State,
    location: Result<&RemoteFilesBrowserLocation, &remote::RemoteError>,
) -> Status<remote::RemoteError> {
    match location {
        Ok(location) => {
            if selectors::select_is_root_loaded(state, &location) {
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
    notify: &store::Notify,
    options: RemoteFilesBrowserOptions,
    location: Result<RemoteFilesBrowserLocation, remote::RemoteError>,
) -> u32 {
    notify(store::Event::RemoteFilesBrowsers);

    let browser_id = state.remote_files_browsers.next_id;

    state.remote_files_browsers.next_id += 1;

    let status = get_initial_status(state, location.as_ref());

    let browser = RemoteFilesBrowser {
        options,
        location: location.ok(),
        status,
        items: Vec::new(),
        selection: Selection::default(),
        sort: Default::default(),
    };

    state
        .remote_files_browsers
        .browsers
        .insert(browser_id, browser);

    update_items(state, notify, browser_id);

    browser_id
}

pub fn destroy(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    notify(store::Event::RemoteFilesBrowsers);

    state.remote_files_browsers.browsers.remove(&browser_id);
}

pub fn loaded(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    location: &RemoteFilesBrowserLocation,
    res: Result<(), remote::RemoteError>,
) {
    let browser = match state.remote_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RemoteFilesBrowsers);

    if browser
        .location
        .as_ref()
        .filter(|loc| loc == &location)
        .is_some()
    {
        match res {
            Ok(()) => browser.status = Status::Loaded,
            Err(err) => browser.status = Status::Error { error: err },
        }
    }

    update_items(state, notify, browser_id);
}

pub fn update_items(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.remote_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let items: Vec<RemoteFilesBrowserItem> = browser
        .location
        .as_ref()
        .map(|location| match location {
            RemoteFilesBrowserLocation::Home => selectors::select_home_items(state),
            RemoteFilesBrowserLocation::Bookmarks => {
                selectors::sort_items(selectors::select_bookmarks_items(state), &browser.sort)
            }
            RemoteFilesBrowserLocation::Files(location) => selectors::sort_items(
                selectors::select_files_items(state, location),
                &browser.sort,
            ),
            RemoteFilesBrowserLocation::Shared => {
                selectors::sort_items(selectors::select_shared_items(state), &browser.sort)
            }
        })
        .unwrap_or(vec![]);

    let item_ids_set: HashSet<String> = items.iter().map(|item| item.id.clone()).collect();

    let browser = match state.remote_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    if browser.items != items {
        browser.items = items;

        notify(store::Event::RemoteFilesBrowsers);
    }

    let select_item_id = if let Some(name) = browser.options.select_name.clone() {
        let item_id = browser.items.iter().find_map(|item| {
            if item.name_lower == name.to_lowercase() {
                Some(item.id.clone())
            } else {
                None
            }
        });

        if matches!(&browser.status, Status::Loaded) || item_id.is_some() {
            browser.options.select_name = None;
        }

        item_id
    } else {
        None
    };

    if selection_mutations::update_selection(&mut browser.selection, item_ids_set) {
        notify(store::Event::RemoteFilesBrowsers);
    }

    if let Some(item_id) = select_item_id {
        select_item(state, notify, browser_id, &item_id, false, false, true);
    }
}

pub fn select_item(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    item_id: &str,
    extend: bool,
    range: bool,
    force: bool,
) {
    let browser = match state.remote_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    let item_ids: Vec<_> = browser.items.iter().map(|item| item.id.clone()).collect();

    let browser = match state.remote_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RemoteFilesBrowsers);

    selection_mutations::select_item(
        &mut browser.selection,
        item_ids,
        item_id,
        extend,
        range,
        force,
    )
}

pub fn select_all(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.remote_files_browsers.browsers.get(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RemoteFilesBrowsers);

    let item_ids: Vec<_> = browser.items.iter().map(|item| item.id.clone()).collect();

    let browser = match state.remote_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    selection_mutations::set_selection(&mut browser.selection, item_ids);
}

pub fn clear_selection(state: &mut store::State, notify: &store::Notify, browser_id: u32) {
    let browser = match state.remote_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RemoteFilesBrowsers);

    selection_mutations::clear_selection(&mut browser.selection);
}

pub fn sort_by(
    state: &mut store::State,
    notify: &store::Notify,
    browser_id: u32,
    field: RemoteFilesSortField,
) {
    let browser = match state.remote_files_browsers.browsers.get_mut(&browser_id) {
        Some(browser) => browser,
        _ => return,
    };

    notify(store::Event::RemoteFilesBrowsers);

    let direction = if browser.sort.field == field {
        browser.sort.direction.clone().reverse()
    } else {
        match field {
            RemoteFilesSortField::Size | RemoteFilesSortField::Modified => SortDirection::Desc,
            _ => SortDirection::Asc,
        }
    };

    browser.sort.field = field;
    browser.sort.direction = direction;

    update_items(state, notify, browser_id);
}

pub fn handle_remote_files_mutation(state: &mut store::State, notify: &store::Notify) {
    for browser_id in state
        .remote_files_browsers
        .browsers
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        update_items(state, notify, browser_id);
    }
}
