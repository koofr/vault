use std::{collections::HashMap, time::Duration};

use crate::{
    cipher::errors::DecryptFilenameError,
    common::state::Status,
    eventstream::state::MountSubscription,
    files::{file_category::FileCategory, files_filter::FilesFilter},
    repo_files::errors::{DeleteFileError, LoadFilesError},
    store::NextId,
    transfers::errors::TransferError,
    types::{DecryptedName, EncryptedName, EncryptedPath, RepoId},
};

use super::errors::SaveError;

#[derive(Debug, PartialEq)]
pub struct RepoFilesDetailsInfo<'a> {
    pub repo_id: Option<&'a RepoId>,
    pub parent_path: Option<EncryptedPath>,
    pub path: Option<&'a EncryptedPath>,
    pub status: Status<LoadFilesError>,
    pub file_name: Option<DecryptedName>,
    pub file_ext: Option<String>,
    pub file_category: Option<FileCategory>,
    pub file_modified: Option<i64>,
    pub file_exists: bool,
    pub content_status: Status<TransferError>,
    pub transfer_id: Option<u32>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesDetailsContentData {
    pub bytes: Vec<u8>,
    pub remote_size: Option<i64>,
    pub remote_modified: Option<i64>,
    pub remote_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesDetailsContentLoading {
    pub remote_size: Option<i64>,
    pub remote_modified: Option<i64>,
    pub remote_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesDetailsContent {
    pub status: Status<TransferError>,
    pub data: Option<RepoFilesDetailsContentData>,
    pub loading: Option<RepoFilesDetailsContentLoading>,
    pub version: u32,
    pub transfer_id: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesDetailsLocation {
    pub repo_id: RepoId,
    pub path: EncryptedPath,
    pub name: EncryptedName,
    pub decrypted_name: Option<Result<DecryptedName, DecryptFilenameError>>,
    pub eventstream_mount_subscription: Option<MountSubscription>,
    pub content: RepoFilesDetailsContent,
    pub is_editing: bool,
    pub is_dirty: bool,
    pub save_status: Status<SaveError>,
    pub delete_status: Status<DeleteFileError>,
    pub should_destroy: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesDetailsOptions {
    pub load_content: FilesFilter,
    pub autosave_interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoFilesDetails {
    pub id: u32,
    pub options: RepoFilesDetailsOptions,
    pub location: Option<RepoFilesDetailsLocation>,
    pub status: Status<LoadFilesError>,
    pub repo_files_subscription_id: u32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RepoFilesDetailsState {
    pub details: HashMap<u32, RepoFilesDetails>,
    pub next_id: NextId,
}

impl RepoFilesDetailsState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}

#[derive(Debug, Clone)]
pub enum SaveInitiator {
    User,
    Autosave,
    Cancel,
}
