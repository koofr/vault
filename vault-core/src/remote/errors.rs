use std::collections::HashMap;

use thiserror::Error;

use crate::{http, user_error::UserError};

use super::models;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiErrorCode {
    NotFound,
    AlreadyExists,
    Conflict,
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
            "Conflict" => Self::Conflict,
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

    pub fn from_api_error_details(
        api_error_details: models::ApiErrorDetails,
        request_id: Option<String>,
    ) -> Self {
        Self::ApiError {
            code: api_error_details.code.as_str().into(),
            message: api_error_details.message,
            request_id,
            extra: api_error_details.extra,
        }
    }

    pub fn is_api_error_code(&self, expected_code: ApiErrorCode) -> bool {
        match &self {
            Self::ApiError { code, .. } => code == &expected_code,
            _ => false,
        }
    }
}

impl From<models::ApiError> for RemoteError {
    fn from(api_error: models::ApiError) -> Self {
        Self::from_api_error_details(api_error.error, Some(api_error.request_id))
    }
}
