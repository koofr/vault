use std::collections::{HashMap, HashSet};

use serde_json::Value;

use crate::store::NextId;

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
    pub id: String,
    pub file_id: Option<String>,
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
    pub open_ids: HashSet<String>,
    pub loading_ids: HashSet<String>,
    pub selected_id: Option<String>,
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
