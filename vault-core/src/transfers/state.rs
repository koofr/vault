use std::collections::HashMap;

use futures::future::BoxFuture;

use crate::{
    common::state::SizeInfo,
    files::{file_category::FileCategory, file_icon::FileIconAttrs},
    repo_files::state::RepoFilesUploadResult,
    repo_files_read::state::RepoFileReader,
    store::NextId,
    types::{DecryptedName, EncryptedName, EncryptedPath, RepoFileId, RepoId, TimeMillis},
};

use super::errors::TransferError;

/// TransferDisplayName is a transfer display name and can be "file.txt" or "path/to/file.txt" (no leading slash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransferDisplayName(pub String);

/// TransferUploadRelativeName is "file.txt" or "path/to/file.txt" (no leading slash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransferUploadRelativeName(pub String);

/// If TransferUploadRelativeName is "path/to/file.txt" then TransferUploadRelativeNamePath
/// is "path/to" (no leading slash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransferUploadRelativeNamePath(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum TransferState {
    Waiting,
    Processing,
    Transferring,
    Failed { error: TransferError },
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadTransfer {
    pub repo_id: RepoId,
    pub parent_path: EncryptedPath,
    pub parent_file_id: RepoFileId,
    pub name_rel_path: Option<TransferUploadRelativeNamePath>,
    pub original_name: DecryptedName,
    pub current_name: DecryptedName,
    pub current_name_encrypted: EncryptedName,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferType {
    Upload(UploadTransfer),
    Download,
    DownloadReader,
}

impl TransferType {
    pub fn upload_transfer(&self) -> Option<&UploadTransfer> {
        match self {
            Self::Upload(upload_transfer) => Some(upload_transfer),
            _ => None,
        }
    }

    pub fn upload_transfer_mut(&mut self) -> Option<&mut UploadTransfer> {
        match self {
            Self::Upload(ref mut upload_transfer) => Some(upload_transfer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transfer {
    pub id: u32,
    pub typ: TransferType,
    pub name: TransferDisplayName,
    pub size: SizeInfo,
    pub category: FileCategory,
    pub started: Option<TimeMillis>,
    pub is_persistent: bool,
    pub is_retriable: bool,
    pub is_openable: bool,
    pub state: TransferState,
    pub transferred_bytes: i64,
    pub attempts: usize,
    pub order: usize,
}

impl Transfer {
    pub fn upload_transfer(&self) -> Option<&UploadTransfer> {
        self.typ.upload_transfer()
    }

    pub fn upload_transfer_mut(&mut self) -> Option<&mut UploadTransfer> {
        self.typ.upload_transfer_mut()
    }

    pub fn file_icon_attrs(&self) -> FileIconAttrs {
        FileIconAttrs {
            category: self.category.clone(),
            is_dl: false,
            is_ul: false,
            is_download_transfer: match &self.typ {
                TransferType::Upload(..) => false,
                TransferType::Download => true,
                TransferType::DownloadReader => true,
            },
            is_upload_transfer: match &self.typ {
                TransferType::Upload(..) => true,
                TransferType::Download => false,
                TransferType::DownloadReader => false,
            },
            is_export: false,
            is_import: false,
            is_android: false,
            is_ios: false,
            is_vault_repo: false,
            is_error: match &self.state {
                TransferState::Failed { .. } => true,
                _ => false,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TransfersState {
    pub transfers: HashMap<u32, Transfer>,
    pub next_id: NextId,
    pub started: Option<TimeMillis>,
    pub last_progress_update: Option<TimeMillis>,
    pub transferring_count: usize,
    pub transferring_uploads_count: usize,
    pub transferring_downloads_count: usize,
    pub done_count: usize,
    pub failed_count: usize,
    pub retriable_count: usize,
    pub total_count: usize,
    pub done_bytes: i64,
    pub failed_bytes: i64,
    pub total_bytes: i64,
}

impl TransfersState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}

#[derive(Debug, Clone)]
pub enum RetryInitiator {
    User,
    Autoretry,
}

pub type UploadResult = Result<RepoFilesUploadResult, TransferError>;

pub type CreateUploadResult = Result<BoxFuture<'static, UploadResult>, TransferError>;

pub type CreateUploadResultFuture = BoxFuture<'static, CreateUploadResult>;

pub type DownloadResult = Result<(), TransferError>;

pub type CreateDownloadResult = Result<BoxFuture<'static, DownloadResult>, TransferError>;

pub type CreateDownloadResultFuture = BoxFuture<'static, CreateDownloadResult>;

pub type DownloadReaderResult = (u32, RepoFileReader);
