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
        RemoteError::from_code(ApiErrorCode::NotDir, "Not a dir")
    }

    pub fn move_into_self() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::MoveIntoSelf, "Cannot move into itself")
    }

    pub fn move_root() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::NotDir, "Cannot move root")
    }

    pub fn invalid_path() -> RemoteError {
        RemoteError::from_code(ApiErrorCode::InvalidPath, "Invalid name or path")
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum GetRepoMountPathError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadFilesError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for LoadFilesError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LoadFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum FileNameError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
}

impl UserError for FileNameError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DecryptFilesError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum UploadFileReaderError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("canceled")]
    Canceled,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for UploadFileReaderError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

impl From<EnsureDirError> for UploadFileReaderError {
    fn from(err: EnsureDirError) -> Self {
        match err {
            EnsureDirError::RepoNotFound(err) => Self::RepoNotFound(err),
            EnsureDirError::RepoLocked(err) => Self::RepoLocked(err),
            EnsureDirError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
            EnsureDirError::Canceled => Self::Canceled,
            EnsureDirError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DeleteFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("canceled")]
    Canceled,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for DeleteFileError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CreateDirError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("canceled")]
    Canceled,
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

impl From<LoadFilesError> for CreateDirError {
    fn from(err: LoadFilesError) -> Self {
        match err {
            LoadFilesError::RepoNotFound(err) => Self::RepoNotFound(err),
            LoadFilesError::RepoLocked(err) => Self::RepoLocked(err),
            LoadFilesError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CreateFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("canceled")]
    Canceled,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for CreateFileError {
    fn user_error(&self) -> String {
        match self {
            Self::RemoteError(RemoteError::ApiError {
                code: ApiErrorCode::AlreadyExists,
                ..
            }) => String::from("File with this name already exists."),
            _ => self.to_string(),
        }
    }
}

impl From<UploadFileReaderError> for CreateFileError {
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
pub enum EnsureDirError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("canceled")]
    Canceled,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl From<CreateDirError> for EnsureDirError {
    fn from(err: CreateDirError) -> Self {
        match err {
            CreateDirError::RepoNotFound(err) => Self::RepoNotFound(err),
            CreateDirError::RepoLocked(err) => Self::RepoLocked(err),
            CreateDirError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
            CreateDirError::Canceled => Self::Canceled,
            CreateDirError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

impl From<LoadFileError> for EnsureDirError {
    fn from(err: LoadFileError) -> Self {
        match err {
            LoadFileError::RepoNotFound(err) => Self::RepoNotFound(err),
            LoadFileError::RepoLocked(err) => Self::RepoLocked(err),
            LoadFileError::RemoteError(err) => Self::RemoteError(err),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum RenameFileError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("rename root")]
    RenameRoot,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for RenameFileError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
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

impl UserError for CopyFileError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum MoveFileError {
    #[error("invalid path")]
    InvalidPath,
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("move root")]
    MoveRoot,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for MoveFileError {
    fn user_error(&self) -> String {
        self.to_string()
    }
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
