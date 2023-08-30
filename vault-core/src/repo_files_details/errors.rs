use thiserror::Error;

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    remote::RemoteError,
    repo_files::errors::{LoadFilesError, UploadFileReaderError},
    repo_files_read::errors::GetFilesReaderError,
    repos::errors::{RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum LoadDetailsError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
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

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum LoadContentError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("file not found")]
    FileNotFound,
    #[error("files empty")]
    FilesEmpty,
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    DecryptSizeError(#[from] DecryptSizeError),
    #[error("already loading")]
    AlreadyLoading,
    #[error("load filter mismatch")]
    LoadFilterMismatch,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
    #[error("{0}")]
    IOError(String),
    #[error("aborted")]
    Aborted,
}

impl From<GetFilesReaderError> for LoadContentError {
    fn from(err: GetFilesReaderError) -> Self {
        match err {
            GetFilesReaderError::RepoNotFound(err) => Self::RepoNotFound(err),
            GetFilesReaderError::RepoLocked(err) => Self::RepoLocked(err),
            GetFilesReaderError::FileNotFound => Self::FileNotFound,
            GetFilesReaderError::FilesEmpty => {
                panic!("invalid state: GetFilesReaderError::FilesEmpty for LoadContentError")
            }
            GetFilesReaderError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
            GetFilesReaderError::DecryptSizeError(err) => Self::DecryptSizeError(err),
            GetFilesReaderError::RemoteError(err) => Self::RemoteError(err),
            GetFilesReaderError::IOError(err) => Self::IOError(err),
            GetFilesReaderError::Aborted => Self::Aborted,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, UserError)]
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
