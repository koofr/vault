use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::store::NextId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DirPickerItemId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DirPickerFileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirPickerItemType {
    Folder,
    Import,
    Export,
    Hosted,
    Desktop,
    DesktopOffline,
    Dropbox,
    Googledrive,
    Onedrive,
    Bookmarks,
    Bookmark,
    Shared,
    Repo,
}

#[derive(Debug, Clone)]
pub struct DirPickerItem {
    pub id: DirPickerItemId,
    pub file_id: Option<DirPickerFileId>,
    pub typ: DirPickerItemType,
    pub is_open: bool,
    pub is_selected: bool,
    pub is_selectable: bool,
    pub is_loading: bool,
    pub spaces: u16,
    pub has_arrow: bool,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct DirPicker {
    pub id: u32,
    pub options: Value,
    pub items: Vec<DirPickerItem>,
    pub open_ids: HashSet<DirPickerItemId>,
    pub loading_ids: HashSet<DirPickerItemId>,
    pub selected_id: Option<DirPickerItemId>,
}

#[derive(Debug, Clone, Default)]
pub struct DirPickersState {
    pub pickers: HashMap<u32, DirPicker>,
    pub next_id: NextId,
}

impl DirPickersState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
