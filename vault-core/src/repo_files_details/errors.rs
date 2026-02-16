use thiserror::Error;

use crate::{
    cipher::errors::DecryptFilenameError,
    intl,
    remote::RemoteError,
    repo_files::errors::{LoadFilesError, UploadFileReaderError},
    repos::errors::{GetCipherError, RepoLockedError, RepoNotFoundError},
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
            Self::RemoteError(err) => err.user_error(intl_service),
        }
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
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
            Self::TransferError(err) => err.user_error(intl_service),
            Self::FileNotFound => intl::format_message!(
                intl_service,
                "core.repo_files_details.file_not_found.error",
                "Error shown in the file details view when the file cannot be found.",
                "File not found"
            ),
            Self::DecryptFilenameError(err) => err.user_error(intl_service),
            Self::AlreadyLoading => self.to_string(),
            Self::LoadFilterMismatch => self.to_string(),
        }
    }
}

impl From<GetCipherError> for LoadContentError {
    fn from(err: GetCipherError) -> Self {
        match err {
            GetCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            GetCipherError::RepoLocked(err) => Self::RepoLocked(err),
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
    #[error("{0}")]
    DecryptDataError(String),
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
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
            Self::DecryptFilenameError(err) => err.user_error(intl_service),
            Self::DecryptDataError(err) => err.clone(),
            Self::AlreadySaving => self.to_string(),
            Self::NotDirty => self.to_string(),
            Self::InvalidState => self.to_string(),
            Self::AutosaveNotPossible => self.to_string(),
            Self::DiscardChanges { .. } => self.to_string(),
            Self::Canceled => self.to_string(),
            Self::CannotSaveRoot => self.to_string(),
            Self::RemoteError(err) => err.user_error(intl_service),
        }
    }
}

impl From<GetCipherError> for SaveError {
    fn from(err: GetCipherError) -> Self {
        match err {
            GetCipherError::RepoNotFound(err) => Self::RepoNotFound(err),
            GetCipherError::RepoLocked(err) => Self::RepoLocked(err),
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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SetContentError {
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
}

impl UserError for SetContentError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoLocked(err) => err.user_error(intl_service),
        }
    }
}
