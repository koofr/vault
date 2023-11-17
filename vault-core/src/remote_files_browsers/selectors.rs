use lazy_static::lazy_static;

use crate::{
    remote_files::{
        selectors as remote_files_selectors,
        state::{
            Mount, MountOrigin, MountType, RemoteFile, RemoteFileType, RemoteFilesSort,
            RemoteFilesSortField,
        },
    },
    selection::{selectors as selection_selectors, state::SelectionSummary},
    store,
    types::{MountId, RemoteFileId, RemoteName, RemoteNameLower, RemotePath},
    utils::path_utils,
};

use super::state::{
    RemoteFilesBrowser, RemoteFilesBrowserBreadcrumb, RemoteFilesBrowserInfo,
    RemoteFilesBrowserItem, RemoteFilesBrowserItemId, RemoteFilesBrowserItemInfo,
    RemoteFilesBrowserItemType, RemoteFilesBrowserLocation, RemoteFilesBrowserLocationFiles,
    RemoteFilesBrowserOptions,
};

lazy_static! {
    pub static ref ITEM_ID_HOME: RemoteFilesBrowserItemId = RemoteFilesBrowserItemId("".into());
    pub static ref ITEM_ID_BOOKMARKS: RemoteFilesBrowserItemId =
        RemoteFilesBrowserItemId("bookmarks".into());
    pub static ref ITEM_ID_SHARED: RemoteFilesBrowserItemId =
        RemoteFilesBrowserItemId("shared".into());
    pub static ref ITEM_ID_PREFIX_BOOKMARKS: RemoteFilesBrowserItemId =
        RemoteFilesBrowserItemId("bookmarks:".into());
    pub static ref ITEM_ID_PREFIX_PLACES: RemoteFilesBrowserItemId =
        RemoteFilesBrowserItemId("places:".into());
    pub static ref ITEM_ID_PREFIX_SHARED: RemoteFilesBrowserItemId =
        RemoteFilesBrowserItemId("shared:".into());
}

pub fn get_file_item_id(
    item_id_prefix: &RemoteFilesBrowserItemId,
    mount_id: &MountId,
    path: &RemotePath,
) -> RemoteFilesBrowserItemId {
    RemoteFilesBrowserItemId(format!("{}{}:{}", item_id_prefix.0, mount_id.0, path.0))
}

pub fn parse_location_files_item_id_prefix(
    location: &RemoteFilesBrowserItemId,
) -> Option<&RemoteFilesBrowserItemId> {
    if location.0.starts_with(&ITEM_ID_PREFIX_BOOKMARKS.0) {
        Some(&ITEM_ID_PREFIX_BOOKMARKS)
    } else if location.0.starts_with(&ITEM_ID_PREFIX_PLACES.0) {
        Some(&ITEM_ID_PREFIX_PLACES)
    } else if location.0.starts_with(&ITEM_ID_PREFIX_SHARED.0) {
        Some(&ITEM_ID_PREFIX_SHARED)
    } else {
        None
    }
}

pub fn parse_location_files(
    location: &RemoteFilesBrowserItemId,
) -> Option<(RemoteFilesBrowserItemId, MountId, RemotePath)> {
    if let Some(item_id_prefix) = parse_location_files_item_id_prefix(location) {
        let mut parts = location.0.splitn(3, ':');

        match (
            parts.next(),
            parts.next(),
            parts.next().map(path_utils::normalize_path),
        ) {
            (_, Some(mount_id), Some(Ok(path))) if mount_id.len() > 0 => Some((
                item_id_prefix.to_owned(),
                MountId(mount_id.to_owned()),
                RemotePath(path),
            )),
            _ => None,
        }
    } else {
        None
    }
}

pub fn get_bookmarks_item() -> RemoteFilesBrowserItem {
    RemoteFilesBrowserItem {
        id: ITEM_ID_BOOKMARKS.to_owned(),
        mount_id: None,
        path: None,
        name: RemoteName("Bookmarks".into()),
        name_lower: RemoteNameLower("bookmarks".into()),
        typ: RemoteFilesBrowserItemType::Bookmarks,
        size: None,
        modified: None,
    }
}

pub fn get_place_item(mount: &Mount, file: &RemoteFile) -> RemoteFilesBrowserItem {
    RemoteFilesBrowserItem {
        id: get_file_item_id(&ITEM_ID_PREFIX_PLACES, &file.mount_id, &file.path),
        mount_id: Some(file.mount_id.clone()),
        path: Some(file.path.clone()),
        name: mount.name.clone(),
        name_lower: mount.name_lower.clone(),
        typ: RemoteFilesBrowserItemType::Place {
            origin: mount.origin.clone(),
        },
        size: None,
        modified: None,
    }
}

pub fn get_file_item(
    item_id_prefix: &RemoteFilesBrowserItemId,
    file: &RemoteFile,
    mount: Option<&Mount>,
) -> RemoteFilesBrowserItem {
    RemoteFilesBrowserItem {
        id: get_file_item_id(item_id_prefix, &file.mount_id, &file.path),
        mount_id: Some(file.mount_id.clone()),
        path: Some(file.path.clone()),
        name: file.name.clone(),
        name_lower: file.name_lower.clone(),
        typ: RemoteFilesBrowserItemType::File {
            item_id_prefix: item_id_prefix.to_owned(),
            typ: file.typ.clone(),
            file_icon_attrs: file.file_icon_attrs(mount),
        },
        size: file.size,
        modified: file.modified,
    }
}

pub fn get_shared_item() -> RemoteFilesBrowserItem {
    RemoteFilesBrowserItem {
        id: ITEM_ID_SHARED.to_owned(),
        mount_id: None,
        path: None,
        name: RemoteName("Shared".into()),
        name_lower: RemoteNameLower("shared".into()),
        typ: RemoteFilesBrowserItemType::Shared,
        size: None,
        modified: None,
    }
}

pub fn sort_items(
    items: Vec<RemoteFilesBrowserItem>,
    sort: &RemoteFilesSort,
) -> Vec<RemoteFilesBrowserItem> {
    let RemoteFilesSort { field, direction } = sort;

    let (mut dirs, mut files): (Vec<_>, Vec<_>) =
        items.into_iter().partition(|item| match &item.typ {
            RemoteFilesBrowserItemType::Bookmarks => true,
            RemoteFilesBrowserItemType::Place { .. } => true,
            RemoteFilesBrowserItemType::File { typ, .. } => matches!(typ, RemoteFileType::Dir),
            RemoteFilesBrowserItemType::Shared => true,
        });

    match field {
        RemoteFilesSortField::Name => {
            dirs.sort_by(|a, b| direction.ordering(a.name_lower.cmp(&b.name_lower)));
            files.sort_by(|a, b| direction.ordering(a.name_lower.cmp(&b.name_lower)));
        }
        RemoteFilesSortField::Size => {
            dirs.sort_by(|a, b| a.name_lower.cmp(&b.name_lower));
            files.sort_by(|a, b| direction.ordering(a.size.cmp(&b.size)));
        }
        RemoteFilesSortField::Modified => {
            dirs.sort_by(|a, b| a.name_lower.cmp(&b.name_lower));
            files.sort_by(|a, b| direction.ordering(a.modified.cmp(&b.modified)));
        }
    }

    dirs.into_iter().chain(files.into_iter()).collect()
}

pub fn get_home_breadcrumb() -> RemoteFilesBrowserBreadcrumb {
    RemoteFilesBrowserBreadcrumb {
        id: ITEM_ID_HOME.clone(),
        mount_id: None,
        path: None,
        name: RemoteName("Koofr".into()),
        last: false,
    }
}

pub fn get_bookmarks_breadcrumb() -> RemoteFilesBrowserBreadcrumb {
    RemoteFilesBrowserBreadcrumb {
        id: ITEM_ID_BOOKMARKS.clone(),
        mount_id: None,
        path: None,
        name: RemoteName("Bookmarks".into()),
        last: false,
    }
}

pub fn get_shared_breadcrumb() -> RemoteFilesBrowserBreadcrumb {
    RemoteFilesBrowserBreadcrumb {
        id: ITEM_ID_SHARED.clone(),
        mount_id: None,
        path: None,
        name: RemoteName("Shared".into()),
        last: false,
    }
}

pub fn select_browser<'a>(
    state: &'a store::State,
    browser_id: u32,
) -> Option<&'a RemoteFilesBrowser> {
    state.remote_files_browsers.browsers.get(&browser_id)
}

pub fn select_browser_location<'a>(
    state: &'a store::State,
    browser_id: u32,
) -> Option<&'a RemoteFilesBrowserLocation> {
    select_browser(state, browser_id).and_then(|browser| browser.location.as_ref())
}

pub fn select_home_items(
    state: &store::State,
    options: &RemoteFilesBrowserOptions,
) -> Vec<RemoteFilesBrowserItem> {
    let mut items = vec![];

    let bookmarks_files = remote_files_selectors::select_bookmarks_files(state);

    if bookmarks_files.len() > 0 {
        items.push(get_bookmarks_item());
    }

    let places_mount_files = remote_files_selectors::select_places_mount_files(state);

    for (mount, file) in places_mount_files {
        if options.only_hosted_devices
            && (mount.typ != MountType::Device || mount.origin != MountOrigin::Hosted)
        {
            continue;
        }

        items.push(get_place_item(mount, file));
    }

    if !options.only_hosted_devices {
        let shared_mount_files = remote_files_selectors::select_shared_mount_files(state);

        if shared_mount_files.len() > 0 {
            items.push(get_shared_item());
        }
    }

    items
}

pub fn select_bookmarks_items(state: &store::State) -> Vec<RemoteFilesBrowserItem> {
    remote_files_selectors::select_bookmarks_files(state)
        .into_iter()
        .map(|file| get_file_item(&ITEM_ID_PREFIX_BOOKMARKS, file, None))
        .collect()
}

pub fn select_files_items(
    state: &store::State,
    location: &RemoteFilesBrowserLocationFiles,
) -> Vec<RemoteFilesBrowserItem> {
    remote_files_selectors::select_files(state, &location.mount_id, &location.path.to_lowercase())
        .into_iter()
        .map(|file| get_file_item(&location.item_id_prefix, file, None))
        .collect()
}

pub fn select_shared_items(state: &store::State) -> Vec<RemoteFilesBrowserItem> {
    remote_files_selectors::select_shared_mount_files(state)
        .into_iter()
        .map(|(mount, file)| get_file_item(&ITEM_ID_PREFIX_SHARED, file, Some(mount)))
        .collect()
}

pub fn select_is_root_loaded(state: &store::State, location: &RemoteFilesBrowserLocation) -> bool {
    match location {
        RemoteFilesBrowserLocation::Home => {
            state.remote_files.places_loaded
                && state.remote_files.bookmarks_loaded
                && state.remote_files.shared_files_loaded
        }
        RemoteFilesBrowserLocation::Bookmarks => state.remote_files.bookmarks_loaded,
        RemoteFilesBrowserLocation::Files(location) => {
            remote_files_selectors::select_is_root_loaded(
                state,
                &location.mount_id,
                &location.path.to_lowercase(),
            )
        }
        RemoteFilesBrowserLocation::Shared => state.remote_files.shared_files_loaded,
    }
}

pub fn select_is_selected(
    state: &store::State,
    browser_id: u32,
    id: &RemoteFilesBrowserItemId,
) -> bool {
    select_browser(state, browser_id)
        .map(|browser| browser.selection.selection.contains(id))
        .unwrap_or(false)
}

pub fn select_selection_summary(state: &store::State, browser_id: u32) -> SelectionSummary {
    select_browser(state, browser_id)
        .map(|browser| {
            selection_selectors::select_selection_summary(&browser.selection, browser.items.len())
        })
        .unwrap_or(SelectionSummary::None)
}

pub fn select_info<'a>(state: &'a store::State, browser_id: u32) -> Option<RemoteFilesBrowserInfo> {
    select_browser(state, browser_id).map(|browser| {
        let breadcrumbs = select_breadcrumbs(state, browser_id);
        let last_breadcrumb: Option<&RemoteFilesBrowserBreadcrumb> = breadcrumbs.last();
        let items = select_items_infos(state, browser_id);
        let get_selected_items = || items.iter().filter(|item| item.is_selected);

        let mount_id = last_breadcrumb.and_then(|breadcrumb| breadcrumb.mount_id.clone());
        let path = last_breadcrumb.and_then(|breadcrumb| breadcrumb.path.clone());
        let title = last_breadcrumb.map(|breadcrumb| breadcrumb.name.clone());

        let total_count = browser.items.len();
        let total_size = browser
            .items
            .iter()
            .map(|item| item.size.unwrap_or(0))
            .sum();

        let selected_count = get_selected_items().count();
        let selected_size = get_selected_items()
            .map(|info| info.item.size.unwrap_or(0))
            .sum();
        let selected_item = get_selected_items()
            .next()
            .filter(|_| selected_count == 1)
            .map(|info| info.item);
        let can_create_dir = mount_id.is_some() && path.is_some();

        RemoteFilesBrowserInfo {
            mount_id,
            path,
            selection_summary: select_selection_summary(state, browser_id),
            sort: browser.sort.clone(),
            status: &browser.status,
            title,
            total_count,
            total_size,
            selected_count,
            selected_size,
            selected_item,
            can_create_dir,
            items,
        }
    })
}

pub fn select_items_infos<'a>(
    state: &'a store::State,
    browser_id: u32,
) -> Vec<RemoteFilesBrowserItemInfo<'a>> {
    select_browser(state, browser_id)
        .map(|browser| {
            browser
                .items
                .iter()
                .map(|item| RemoteFilesBrowserItemInfo {
                    item,
                    is_selected: browser.selection.selection.contains(&item.id),
                })
                .collect()
        })
        .unwrap_or_else(|| vec![])
}

pub fn select_breadcrumbs(
    state: &store::State,
    browser_id: u32,
) -> Vec<RemoteFilesBrowserBreadcrumb> {
    let mut breadcrumbs = vec![get_home_breadcrumb()];

    if let Some(location) = select_browser_location(state, browser_id) {
        match location {
            RemoteFilesBrowserLocation::Home => {}
            RemoteFilesBrowserLocation::Bookmarks => breadcrumbs.push(get_bookmarks_breadcrumb()),
            RemoteFilesBrowserLocation::Files(location) => match &location.item_id_prefix {
                prefix
                    if prefix == &*ITEM_ID_PREFIX_BOOKMARKS
                        || prefix == &*ITEM_ID_PREFIX_PLACES =>
                {
                    for breadcrumb in remote_files_selectors::select_breadcrumbs(
                        state,
                        &location.mount_id,
                        &location.path,
                    ) {
                        breadcrumbs.push(RemoteFilesBrowserBreadcrumb {
                            id: get_file_item_id(
                                &location.item_id_prefix,
                                &breadcrumb.mount_id,
                                &breadcrumb.path,
                            ),
                            mount_id: Some(breadcrumb.mount_id),
                            path: Some(breadcrumb.path),
                            name: breadcrumb.name,
                            last: breadcrumb.last,
                        });
                    }
                }
                prefix if prefix == &*ITEM_ID_PREFIX_SHARED => {
                    breadcrumbs.push(get_shared_breadcrumb());
                }
                _ => {}
            },
            RemoteFilesBrowserLocation::Shared => breadcrumbs.push(get_shared_breadcrumb()),
        }
    }

    breadcrumbs.last_mut().unwrap().last = true;

    breadcrumbs
}

pub fn select_root_file_id(state: &store::State, browser_id: u32) -> Option<RemoteFileId> {
    select_browser_location(state, browser_id).and_then(|loc| match loc {
        RemoteFilesBrowserLocation::Files(location) => Some(remote_files_selectors::get_file_id(
            &location.mount_id,
            &location.path.to_lowercase(),
        )),
        _ => None,
    })
}

pub fn select_root_file<'a>(state: &'a store::State, browser_id: u32) -> Option<&'a RemoteFile> {
    select_root_file_id(state, browser_id)
        .and_then(|file_id| remote_files_selectors::select_file(state, &file_id))
}
