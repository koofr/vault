use thiserror::Error;

use crate::{
    remote::{ApiErrorCode, RemoteError},
    user_error::UserError,
};

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

#[derive(Error, Debug, Clone)]
pub enum CreateDirError {
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
