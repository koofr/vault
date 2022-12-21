use crate::remote::{ApiErrorCode, RemoteError};

pub struct RemoteFilesErrors;

impl RemoteFilesErrors {
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
