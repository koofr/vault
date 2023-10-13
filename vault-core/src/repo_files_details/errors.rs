use thiserror::Error;

use crate::{
    cipher::errors::DecryptFilenameError,
    remote::RemoteError,
    repo_files::errors::{LoadFilesError, UploadFileReaderError},
    repos::errors::{RepoLockedError, RepoNotFoundError},
    transfers::errors::TransferError,
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadDetailsError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for LoadDetailsError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

impl From<LoadFilesError> for LoadDetailsError {
    fn from(err: LoadFilesError) -> Self {
        match err {
            LoadFilesError::RepoNotFound(err) => Self::RepoNotFound(err),
            LoadFilesError::RepoLocked(err) => Self::RepoLocked(err),
            LoadFilesError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadContentError {
    #[error("{0}")]
    TransferError(#[from] TransferError),
    #[error("file not found")]
    FileNotFound,
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("already loading")]
    AlreadyLoading,
    #[error("load filter mismatch")]
    LoadFilterMismatch,
}

impl UserError for LoadContentError {
    fn user_error(&self) -> String {
        match self {
            Self::DecryptFilenameError(err) => err.user_error(),
            _ => self.to_string(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SaveError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("already saving")]
    AlreadySaving,
    #[error("not dirty")]
    NotDirty,
    #[error("invalid state")]
    InvalidState,
    #[error("autosave not possible")]
    AutosaveNotPossible,
    #[error("discard changes")]
    DiscardChanges { should_destroy: bool },
    #[error("canceled")]
    Canceled,
    #[error("cannot save root")]
    CannotSaveRoot,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for SaveError {
    fn user_error(&self) -> String {
        match self {
            Self::DecryptFilenameError(err) => err.user_error(),
            _ => self.to_string(),
        }
    }
}

impl From<LoadFilesError> for SaveError {
    fn from(err: LoadFilesError) -> Self {
        match err {
            LoadFilesError::RepoNotFound(err) => Self::RepoNotFound(err),
            LoadFilesError::RepoLocked(err) => Self::RepoLocked(err),
            LoadFilesError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

impl From<UploadFileReaderError> for SaveError {
    fn from(err: UploadFileReaderError) -> Self {
        match err {
            UploadFileReaderError::RepoNotFound(err) => Self::RepoNotFound(err),
            UploadFileReaderError::RepoLocked(err) => Self::RepoLocked(err),
            UploadFileReaderError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
            UploadFileReaderError::Canceled => Self::Canceled,
            UploadFileReaderError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}
