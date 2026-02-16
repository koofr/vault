use thiserror::Error;

use crate::{
    intl,
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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum CreateDirError {
    #[error("canceled")]
    Canceled,
    #[error("{0}")]
    RemoteError(#[from] RemoteError),
}

impl UserError for CreateDirError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::Canceled => self.to_string(),
            Self::RemoteError(RemoteError::ApiError {
                code: ApiErrorCode::AlreadyExists,
                ..
            }) => intl::format_message!(
                intl_service,
                "core.files.create_dir_already_exists.error",
                "Error shown when creating a folder and the name already exists.",
                "Folder with this name already exists."
            ),
            Self::RemoteError(err) => err.user_error(intl_service),
        }
    }
}
