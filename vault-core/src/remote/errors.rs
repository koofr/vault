use std::collections::HashMap;

use thiserror::Error;

use crate::http;
use crate::user_error::UserError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiErrorCode {
    NotFound,
    AlreadyExists,
    NotDir,
    InvalidPath,
    VaultReposLocationNotFound,
    VaultReposAlreadyExists,
    VaultReposMountNotAllowed,
    VaultReposMaxTotalLimitExceeded,
    Other(String),
}

impl From<&str> for ApiErrorCode {
    fn from(code: &str) -> Self {
        match code {
            "NotFound" => Self::NotFound,
            "AlreadyExists" => Self::AlreadyExists,
            "NotDir" => Self::NotDir,
            "InvalidPath" => Self::InvalidPath,
            "VaultReposLocationNotFound" => Self::VaultReposLocationNotFound,
            "VaultReposAlreadyExists" => Self::VaultReposAlreadyExists,
            "VaultReposMountNotAllowed" => Self::VaultReposMountNotAllowed,
            "VaultReposMaxTotalLimitExceeded" => Self::VaultReposMaxTotalLimitExceeded,
            _ => Self::Other(code.to_owned()),
        }
    }
}

#[derive(Error, Debug, Clone, UserError, PartialEq, Eq)]
pub enum RemoteError {
    #[error("{message}")]
    ApiError {
        code: ApiErrorCode,
        message: String,
        request_id: Option<String>,
        extra: Option<HashMap<String, serde_json::Value>>,
    },
    #[error("{0}")]
    HttpError(#[from] http::HttpError),
}

impl RemoteError {
    pub fn from_code(code: ApiErrorCode, message: &str) -> Self {
        Self::ApiError {
            code,
            message: message.to_owned(),
            request_id: None,
            extra: None,
        }
    }
}
