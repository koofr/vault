use thiserror::Error;

use crate::{
    cipher::errors::DecryptFilenameError,
    remote::{ApiErrorCode, RemoteError},
    repos::errors::{RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

pub struct RepoFilesErrors;

impl RepoFilesErrors {
    pub fn not_found() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::NotFound, "Not found")
    }

    pub fn already_exists() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::AlreadyExists, "Already exists")
    }

    pub fn not_a_dir() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::InvalidPath, "Not a dir")
    }

    pub fn invalid_path() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::InvalidPath, "Invalid name or path")
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RepoMountPathToPathError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
}

#[derive(Error, Debug, Clone)]
pub enum GetRepoMountPathError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
}

#[derive(Error, Debug, Clone, UserError)]
pub enum LoadFilesError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone)]
pub enum LoadFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone)]
pub enum DecryptFilesError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
}

#[derive(Error, Debug, Clone)]
pub enum UploadFileReaderError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone, UserError)]
pub enum DeleteFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone)]
pub enum CreateDirError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for CreateDirError {
    fn user_error(&self) -> String {
        match self {
            Self::RemoteError(RemoteError::ApiError {
                code: ApiErrorCode::AlreadyExists,
                ..
            }) => String::from("Folder with this name already exists."),
            _ => self.to_string(),
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum EnsureDirError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone, UserError)]
pub enum RenameFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone, UserError)]
pub enum CopyFileError {
    #[error("invalid path")]
    InvalidPath,
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone, UserError)]
pub enum MoveFileError {
    #[error("invalid path")]
    InvalidPath,
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl From<CopyFileError> for MoveFileError {
    fn from(err: CopyFileError) -> Self {
        match err {
            CopyFileError::InvalidPath => Self::InvalidPath,
            CopyFileError::RepoNotFound(err) => Self::RepoNotFound(err),
            CopyFileError::RepoLocked(err) => Self::RepoLocked(err),
            CopyFileError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
            CopyFileError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}
