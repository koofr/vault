use std::collections::HashMap;

use crate::{
    common::state::Status,
    eventstream::state::MountSubscription,
    repo_files::{
        errors::LoadFilesError,
        state::{RepoFile, RepoFilesBreadcrumb, RepoFilesSort},
    },
    repos::errors::RepoInfoError,
    selection::state::{Selection, SelectionSummary},
    store::NextId,
    types::{DecryptedName, EncryptedPath, RepoFileId, RepoId},
};

#[derive(Debug, PartialEq)]
pub struct RepoFilesBrowserItem<'a> {
    pub file: &'a RepoFile,
    pub is_selected: bool,
}

#[derive(Debug, PartialEq)]
pub struct RepoFilesBrowserInfo<'a> {
    pub repo_id: Option<&'a RepoId>,
    pub path: Option<&'a EncryptedPath>,
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
    pub breadcrumbs: Option<&'a [RepoFilesBreadcrumb]>,
    pub repo_status: Status<RepoInfoError>,
    pub is_locked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepoFilesBrowserLocation {
    Storage {
        repo_id: RepoId,
        path: EncryptedPath,
        eventstream_mount_subscription: Option<MountSubscription>,
    },
}

impl RepoFilesBrowserLocation {
    pub fn repo_id(&self) -> &RepoId {
        match self {
            Self::Storage { repo_id, .. } => repo_id,
        }
    }

    pub fn eq_ignore_transient(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Storage {
                    repo_id: self_repo_id,
                    path: self_path,
                    ..
                },
                Self::Storage {
                    repo_id: other_repo_id,
                    path: other_path,
                    ..
                },
            ) => self_repo_id == other_repo_id && self_path == other_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepoFilesBrowserSource {
    Storage {
        repo_id: RepoId,
        encrypted_path: EncryptedPath,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesBrowserOptions {
    pub select_name: Option<DecryptedName>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesBrowser {
    pub id: u32,
    pub options: RepoFilesBrowserOptions,
    pub location: Option<RepoFilesBrowserLocation>,
    pub status: Status<LoadFilesError>,
    pub breadcrumbs: Option<Vec<RepoFilesBreadcrumb>>,
    pub file_ids: Vec<RepoFileId>,
    pub selection: Selection<RepoFileId>,
    pub sort: RepoFilesSort,
    pub repo_status: Status<RepoInfoError>,
    pub is_locked: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RepoFilesBrowsersState {
    pub browsers: HashMap<u32, RepoFilesBrowser>,
    pub next_id: NextId,
    pub last_sort: RepoFilesSort,
}

impl RepoFilesBrowsersState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
