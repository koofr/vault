use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    files::file_icon::FileIconAttrs,
    remote,
    remote_files::state::{MountOrigin, RemoteFileType, RemoteFilesSort},
    selection::state::{Selection, SelectionSummary},
    store::NextId,
    types::{MountId, RemoteName, RemoteNameLower, RemotePath},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct RemoteFilesBrowserItemId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteFilesBrowserItemType {
    Bookmarks,
    Place {
        origin: MountOrigin,
    },
    File {
        item_id_prefix: RemoteFilesBrowserItemId,
        typ: RemoteFileType,
        file_icon_attrs: FileIconAttrs,
    },
    Shared,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteFilesBrowserItem {
    pub id: RemoteFilesBrowserItemId,
    pub mount_id: Option<MountId>,
    pub path: Option<RemotePath>,
    pub name: RemoteName,
    pub name_lower: RemoteNameLower,
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
    pub mount_id: Option<MountId>,
    pub path: Option<RemotePath>,
    pub selection_summary: SelectionSummary,
    pub sort: RemoteFilesSort,
    pub status: &'a Status<remote::RemoteError>,
    pub title: Option<RemoteName>,
    pub total_count: usize,
    pub total_size: i64,
    pub selected_count: usize,
    pub selected_size: i64,
    pub selected_item: Option<&'a RemoteFilesBrowserItem>,
    pub can_create_dir: bool,
    pub items: Vec<RemoteFilesBrowserItemInfo<'a>>,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowserLocationFiles {
    pub item_id_prefix: RemoteFilesBrowserItemId,
    pub mount_id: MountId,
    pub path: RemotePath,
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
    pub select_name: Option<RemoteName>,
    pub only_hosted_devices: bool,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowser {
    pub options: RemoteFilesBrowserOptions,
    pub location: Option<RemoteFilesBrowserLocation>,
    pub status: Status<remote::RemoteError>,
    pub items: Vec<RemoteFilesBrowserItem>,
    pub selection: Selection<RemoteFilesBrowserItemId>,
    pub sort: RemoteFilesSort,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBrowserBreadcrumb {
    pub id: RemoteFilesBrowserItemId,
    pub mount_id: Option<MountId>,
    pub path: Option<RemotePath>,
    pub name: RemoteName,
    pub last: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFilesBrowsersState {
    pub browsers: HashMap<u32, RemoteFilesBrowser>,
    pub next_id: NextId,
}

impl RemoteFilesBrowsersState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
