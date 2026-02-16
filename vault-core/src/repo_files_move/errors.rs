use thiserror::Error;

use crate::{
    cipher::errors::DecryptFilenameError,
    intl,
    remote::RemoteError,
    repo_files::errors::LoadFilesError,
    repos::errors::{RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ShowError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
    #[error("files empty")]
    FilesEmpty,
}

impl UserError for ShowError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
            Self::RemoteError(err) => err.user_error(intl_service),
            Self::FilesEmpty => self.to_string(),
        }
    }
}

impl From<LoadFilesError> for ShowError {
    fn from(err: LoadFilesError) -> Self {
        match err {
            LoadFilesError::RepoNotFound(err) => Self::RepoNotFound(err),
            LoadFilesError::RepoLocked(err) => Self::RepoLocked(err),
            LoadFilesError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DirPickerClickError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for DirPickerClickError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::RepoNotFound(err) => err.user_error(intl_service),
            Self::RepoLocked(err) => err.user_error(intl_service),
            Self::DecryptFilenameError(err) => err.user_error(intl_service),
            Self::RemoteError(err) => err.user_error(intl_service),
        }
    }
}

impl From<LoadFilesError> for DirPickerClickError {
    fn from(err: LoadFilesError) -> Self {
        match err {
            LoadFilesError::RepoNotFound(err) => Self::RepoNotFound(err),
            LoadFilesError::RepoLocked(err) => Self::RepoLocked(err),
            LoadFilesError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}
