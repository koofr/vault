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
    MoveIntoSelf,
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
            "MoveIntoSelf" => Self::MoveIntoSelf,
            "InvalidPath" => Self::InvalidPath,
            "VaultReposLocationNotFound" => Self::VaultReposLocationNotFound,
            "VaultReposAlreadyExists" => Self::VaultReposAlreadyExists,
            "VaultReposMountNotAllowed" => Self::VaultReposMountNotAllowed,
            "VaultReposMaxTotalLimitExceeded" => Self::VaultReposMaxTotalLimitExceeded,
            _ => Self::Other(code.to_owned()),
        }
    }
}

impl Into<String> for ApiErrorCode {
    fn into(self) -> String {
        match self {
            Self::NotFound => "NotFound".into(),
            Self::AlreadyExists => "AlreadyExists".into(),
            Self::Conflict => "Conflict".into(),
            Self::NotDir => "NotDir".into(),
            Self::MoveIntoSelf => "MoveIntoSelf".into(),
            Self::InvalidPath => "InvalidPath".into(),
            Self::VaultReposLocationNotFound => "VaultReposLocationNotFound".into(),
            Self::VaultReposAlreadyExists => "VaultReposAlreadyExists".into(),
            Self::VaultReposMountNotAllowed => "VaultReposMountNotAllowed".into(),
            Self::VaultReposMaxTotalLimitExceeded => "VaultReposMaxTotalLimitExceeded".into(),
            Self::Other(code) => code.clone(),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, UserError)]
pub enum RemoteError {
    #[error("{message}")]
    ApiError {
        code: ApiErrorCode,
        message: String,
        request_id: Option<String>,
        extra: Option<HashMap<String, serde_json::Value>>,
        status_code: Option<u16>,
    },
    #[error("unexpected status: {status_code}: {message}")]
    UnexpectedStatus { status_code: u16, message: String },
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
            status_code: None,
        }
    }

    pub fn from_api_error(api_error: models::ApiError, status_code: u16) -> Self {
        Self::from_api_error_details(
            api_error.error,
            Some(api_error.request_id),
            Some(status_code),
        )
    }

    pub fn from_api_error_details(
        api_error_details: models::ApiErrorDetails,
        request_id: Option<String>,
        status_code: Option<u16>,
    ) -> Self {
        Self::ApiError {
            code: api_error_details.code.as_str().into(),
            message: api_error_details.message,
            request_id,
            extra: api_error_details.extra,
            status_code,
        }
    }

    pub fn is_api_error_code(&self, expected_code: ApiErrorCode) -> bool {
        match &self {
            Self::ApiError { code, .. } => code == &expected_code,
            _ => false,
        }
    }
}
