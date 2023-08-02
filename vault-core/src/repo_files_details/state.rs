use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    common::state::Status,
    eventstream::service::MountSubscription,
    files::{file_category::FileCategory, files_filter::FilesFilter},
    repo_files::errors::{DeleteFileError, LoadFilesError},
    repo_files_read::errors::GetFilesReaderError,
    store::NextId,
};

use super::errors::SaveError;

pub struct RepoFilesDetailsInfo<'a> {
    pub repo_id: Option<&'a str>,
    pub parent_path: Option<&'a str>,
    pub path: Option<&'a str>,
    pub status: Status<LoadFilesError>,
    pub file_name: Option<&'a str>,
    pub file_ext: Option<String>,
    pub file_category: Option<FileCategory>,
    pub file_modified: Option<i64>,
    pub file_exists: bool,
    pub content_status: Status<GetFilesReaderError>,
    pub save_status: Status<SaveError>,
    pub error: Option<String>,
    pub is_editing: bool,
    pub is_dirty: bool,
    pub should_destroy: bool,
    pub can_save: bool,
    pub can_download: bool,
    pub can_copy: bool,
    pub can_move: bool,
    pub can_delete: bool,
}

#[derive(Debug, Clone)]
pub struct RepoFilesDetailsContentData {
    pub bytes: Vec<u8>,
    pub remote_size: Option<i64>,
    pub remote_modified: Option<i64>,
    pub remote_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepoFilesDetailsContentLoading {
    pub remote_size: Option<i64>,
    pub remote_modified: Option<i64>,
    pub remote_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepoFilesDetailsContent {
    pub status: Status<GetFilesReaderError>,
    pub data: Option<RepoFilesDetailsContentData>,
    pub loading: Option<RepoFilesDetailsContentLoading>,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct RepoFilesDetailsLocation {
    pub repo_id: String,
    pub path: String,
    pub eventstream_mount_subscription: Option<Arc<MountSubscription>>,
    pub content: RepoFilesDetailsContent,
    pub is_editing: bool,
    pub is_dirty: bool,
    pub save_status: Status<SaveError>,
    pub delete_status: Status<DeleteFileError>,
    pub should_destroy: bool,
}

#[derive(Debug, Clone)]
pub struct RepoFilesDetailsOptions {
    pub load_content: FilesFilter,
    pub autosave_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct RepoFilesDetails {
    pub options: RepoFilesDetailsOptions,
    pub location: Option<RepoFilesDetailsLocation>,
    pub status: Status<LoadFilesError>,
    pub repo_files_subscription_id: u32,
}

#[derive(Debug, Clone, Default)]
pub struct RepoFilesDetailsState {
    pub details: HashMap<u32, RepoFilesDetails>,
    pub next_id: NextId,
}

#[derive(Debug, Clone)]
pub enum SaveInitiator {
    User,
    Autosave,
    Cancel,
}
