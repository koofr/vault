use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    repo_files::{
        errors::LoadFilesError,
        state::{RepoFile, RepoFilesSort},
    },
    selection::state::{Selection, SelectionSummary},
};

pub struct RepoFilesBrowserItem<'a> {
    pub file: &'a RepoFile,
    pub is_selected: bool,
}

pub struct RepoFilesBrowserInfo<'a> {
    pub repo_id: Option<&'a str>,
    pub path: Option<&'a str>,
    pub selection_summary: SelectionSummary,
    pub sort: RepoFilesSort,
    pub status: &'a Status<LoadFilesError>,
    pub title: Option<String>,
    pub total_count: usize,
    pub total_size: i64,
    pub selected_count: usize,
    pub selected_size: i64,
    pub selected_file: Option<&'a RepoFile>,
    pub can_download_selected: bool,
    pub can_copy_selected: bool,
    pub can_move_selected: bool,
    pub can_delete_selected: bool,
}

#[derive(Debug, Clone)]
pub struct RepoFilesBrowserLocation {
    pub repo_id: String,
    pub path: String,
    pub eventstream_mount_subscription: Option<Arc<MountSubscription>>,
}

#[derive(Debug, Clone)]
pub struct RepoFilesBrowserOptions {
    pub select_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepoFilesBrowser {
    pub options: RepoFilesBrowserOptions,
    pub location: Option<RepoFilesBrowserLocation>,
    pub status: Status<LoadFilesError>,
    pub file_ids: Vec<String>,
    pub selection: Selection,
    pub sort: RepoFilesSort,
}

#[derive(Debug, Clone, Default)]
pub struct RepoFilesBrowsersState {
    pub browsers: HashMap<u32, RepoFilesBrowser>,
    pub next_id: u32,
}
