use std::collections::{HashMap, HashSet};

use serde_json::Value;

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone)]
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

#[derive(Clone, Default)]
pub struct DirPicker {
    pub id: u32,
    pub options: Value,
    pub items: Vec<DirPickerItem>,
    pub open_ids: HashSet<String>,
    pub loading_ids: HashSet<String>,
    pub selected_id: Option<String>,
}

#[derive(Clone, Default)]
pub struct DirPickersState {
    pub pickers: HashMap<u32, DirPicker>,
    pub next_id: u32,
}
