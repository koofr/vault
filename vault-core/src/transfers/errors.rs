use thiserror::Error;

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    remote::RemoteError,
    repo_files::errors::{FileNameError, LoadFilesError, UploadFileReaderError},
    repo_files_read::errors::GetFilesReaderError,
    repos::errors::{RepoLockedError, RepoNotFoundError},
    user_error::UserError,
};

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum UploadableError {
    #[error("{0}")]
    LocalFileError(String),
    #[error("upload not retriable")]
    NotRetriable,
}

impl From<std::io::Error> for UploadableError {
    fn from(err: std::io::Error) -> Self {
        UploadableError::LocalFileError(err.to_string())
    }
}

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum DownloadableError {
    #[error("{0}")]
    LocalFileError(String),
    #[error("download not openable")]
    NotOpenable,
    #[error("download not retriable")]
    NotRetriable,
}

impl From<std::io::Error> for DownloadableError {
    fn from(err: std::io::Error) -> Self {
        DownloadableError::LocalFileError(err.to_string())
    }
}

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum TransferError {
    #[error("{0}")]
    RepoNotFound(#[from] RepoNotFoundError),
    #[error("{0}")]
    RepoLocked(#[from] RepoLockedError),
    #[error("{0}")]
    DecryptFilenameError(#[from] DecryptFilenameError),
    #[error("{0}")]
    DecryptSizeError(#[from] DecryptSizeError),
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
    #[error("{0}")]
    LocalFileError(String),
    #[error("transfer not retriable")]
    NotRetriable,
    #[error("transfer not openable")]
    NotOpenable,
    #[error("remote file not found")]
    RemoteFileNotFound,
    #[error("remote files empty")]
    RemoteFilesEmpty,
    #[error("transfer not found")]
    TransferNotFound,
    #[error("already exists")]
    AlreadyExists,
    #[error("{0}")]
    IOError(String),
    #[error("aborted")]
    Aborted,
}

impl From<UploadFileReaderError> for TransferError {
    fn from(err: UploadFileReaderError) -> Self {
        match err {
            UploadFileReaderError::RepoNotFound(err) => TransferError::RepoNotFound(err),
            UploadFileReaderError::RepoLocked(err) => TransferError::RepoLocked(err),
            UploadFileReaderError::DecryptFilenameError(err) => {
                TransferError::DecryptFilenameError(err)
            }
            UploadFileReaderError::Canceled => TransferError::Aborted,
            UploadFileReaderError::RemoteError(err) => TransferError::RemoteError(err),
        }
    }
}

impl From<FileNameError> for TransferError {
    fn from(err: FileNameError) -> Self {
        match err {
            FileNameError::RepoNotFound(err) => Self::RepoNotFound(err),
            FileNameError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
        }
    }
}

impl From<GetFilesReaderError> for TransferError {
    fn from(err: GetFilesReaderError) -> Self {
        match err {
            GetFilesReaderError::RepoNotFound(err) => Self::RepoNotFound(err),
            GetFilesReaderError::RepoLocked(err) => Self::RepoLocked(err),
            GetFilesReaderError::FileNotFound => Self::RemoteFileNotFound,
            GetFilesReaderError::FilesEmpty => Self::RemoteFilesEmpty,
            GetFilesReaderError::DecryptFilenameError(err) => Self::DecryptFilenameError(err),
            GetFilesReaderError::DecryptSizeError(err) => Self::DecryptSizeError(err),
            GetFilesReaderError::RemoteError(err) => Self::RemoteError(err),
            GetFilesReaderError::IOError(err) => Self::IOError(err),
            GetFilesReaderError::Aborted => Self::Aborted,
        }
    }
}

impl From<LoadFilesError> for TransferError {
    fn from(err: LoadFilesError) -> Self {
        match err {
            LoadFilesError::RepoNotFound(err) => TransferError::RepoNotFound(err),
            LoadFilesError::RepoLocked(err) => TransferError::RepoLocked(err),
            LoadFilesError::RemoteError(err) => TransferError::RemoteError(err),
        }
    }
}

impl From<UploadableError> for TransferError {
    fn from(err: UploadableError) -> Self {
        match err {
            UploadableError::LocalFileError(err) => TransferError::LocalFileError(err),
            UploadableError::NotRetriable => TransferError::NotRetriable,
        }
    }
}

impl From<DownloadableError> for TransferError {
    fn from(err: DownloadableError) -> Self {
        match err {
            DownloadableError::LocalFileError(err) => TransferError::LocalFileError(err),
            DownloadableError::NotOpenable => TransferError::NotOpenable,
            DownloadableError::NotRetriable => TransferError::NotRetriable,
        }
    }
}

impl From<&std::io::Error> for TransferError {
    fn from(err: &std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::Interrupted => TransferError::Aborted,
            _ => TransferError::IOError(err.to_string()),
        }
    }
}
