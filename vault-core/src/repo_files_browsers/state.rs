use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    repo_files::{
        errors::LoadFilesError,
        state::{RepoFile, RepoFilesBreadcrumb, RepoFilesSort},
    },
    selection::state::{Selection, SelectionSummary},
    store::NextId,
};

#[derive(Debug, PartialEq)]
pub struct RepoFilesBrowserItem<'a> {
    pub file: &'a RepoFile,
    pub is_selected: bool,
}

#[derive(Debug, PartialEq)]
pub struct RepoFilesBrowserInfo<'a> {
    pub repo_id: Option<&'a str>,
    pub path: Option<&'a str>,
    pub selection_summary: SelectionSummary,
    pub sort: RepoFilesSort,
    pub status: Status<LoadFilesError>,
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
    pub items: Vec<RepoFilesBrowserItem<'a>>,
    pub breadcrumbs: Vec<RepoFilesBreadcrumb>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesBrowserLocation {
    pub repo_id: String,
    pub path: String,
    pub eventstream_mount_subscription: Option<Arc<MountSubscription>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesBrowserOptions {
    pub select_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesBrowser {
    pub options: RepoFilesBrowserOptions,
    pub location: Option<RepoFilesBrowserLocation>,
    pub status: Status<LoadFilesError>,
    pub file_ids: Vec<String>,
    pub selection: Selection,
    pub sort: RepoFilesSort,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RepoFilesBrowsersState {
    pub browsers: HashMap<u32, RepoFilesBrowser>,
    pub next_id: NextId,
}

impl RepoFilesBrowsersState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
