use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    files::file_icon::FileIconAttrs,
    remote,
    remote_files::state::{MountOrigin, RemoteFileType, RemoteFilesBreadcrumb, RemoteFilesSort},
    selection::state::{Selection, SelectionSummary},
};

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteFilesBrowserItemType {
    Bookmarks,
    Place {
        origin: MountOrigin,
    },
    File {
        item_id_prefix: String,
        typ: RemoteFileType,
        file_icon_attrs: FileIconAttrs,
    },
    Shared,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFilesBrowserItem {
    pub id: String,
    pub mount_id: Option<String>,
    pub path: Option<String>,
    pub name: String,
    pub name_lower: String,
    pub typ: RemoteFilesBrowserItemType,
    pub size: Option<i64>,
    pub modified: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFilesBrowserItemInfo<'a> {
    pub item: &'a RemoteFilesBrowserItem,
    pub is_selected: bool,
}

pub struct RemoteFilesBrowserInfo<'a> {
    pub mount_id: Option<String>,
    pub path: Option<String>,
    pub selection_summary: SelectionSummary,
    pub sort: RemoteFilesSort,
    pub status: &'a Status<remote::RemoteError>,
    pub title: Option<String>,
    pub total_count: usize,
    pub total_size: i64,
    pub selected_count: usize,
    pub selected_size: i64,
    pub selected_item: Option<&'a RemoteFilesBrowserItem>,
    pub can_create_dir: bool,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowserLocationFiles {
    pub item_id_prefix: String,
    pub mount_id: String,
    pub path: String,
    pub eventstream_mount_subscription: Arc<MountSubscription>,
}

impl PartialEq for RemoteFilesBrowserLocationFiles {
    fn eq(&self, other: &Self) -> bool {
        self.mount_id == other.mount_id && self.path == other.path
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteFilesBrowserLocation {
    Home,
    Bookmarks,
    Files(RemoteFilesBrowserLocationFiles),
    Shared,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowserOptions {
    pub select_name: Option<String>,
    pub only_hosted_devices: bool,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowser {
    pub options: RemoteFilesBrowserOptions,
    pub location: Option<RemoteFilesBrowserLocation>,
    pub status: Status<remote::RemoteError>,
    pub items: Vec<RemoteFilesBrowserItem>,
    pub selection: Selection,
    pub sort: RemoteFilesSort,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowserBreadcrumb {
    pub id: String,
    pub mount_id: Option<String>,
    pub path: Option<String>,
    pub name: String,
    pub last: bool,
}

impl From<RemoteFilesBreadcrumb> for RemoteFilesBrowserBreadcrumb {
    fn from(breadcrumb: RemoteFilesBreadcrumb) -> Self {
        Self {
            id: breadcrumb.id,
            mount_id: Some(breadcrumb.mount_id),
            path: Some(breadcrumb.path),
            name: breadcrumb.name,
            last: breadcrumb.last,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFilesBrowsersState {
    pub browsers: HashMap<u32, RemoteFilesBrowser>,
    pub next_id: u32,
}
