use std::sync::Arc;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use thiserror::Error;
use uuid::Uuid;
use vault_core::remote::models;

#[derive(Debug, Clone, PartialEq)]
pub enum ApiErrorCode {
    NotFound,
    AlreadyExists,
    Conflict,
    BadRequest,
    NotDir,
    NotFile,
    MoveIntoSelf,
    CopyIntoSelf,
    InvalidPath,
    VaultReposLocationNotFound,
    VaultReposAlreadyExists,
    VaultReposMountNotAllowed,
    VaultReposMaxTotalLimitExceeded,
    Other,
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum FakeRemoteError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{0:?}: {1:?}: {2}")]
    ApiError(StatusCode, ApiErrorCode, String),
}

impl IntoResponse for FakeRemoteError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED.into_response(),
            Self::BadRequest(message) => {
                api_error_response(StatusCode::BAD_REQUEST, ApiErrorCode::BadRequest, message)
            }
            Self::ApiError(status_code, code, message) => {
                api_error_response(status_code, code, message)
            }
        }
    }
}

fn api_error_response(status_code: StatusCode, code: ApiErrorCode, message: String) -> Response {
    (
        status_code,
        Json(models::ApiError {
            error: models::ApiErrorDetails {
                code: format!("{:?}", code),
                message,
                extra: None,
            },
            request_id: Uuid::new_v4().to_string(),
        }),
    )
        .into_response()
}

#[derive(Error, Debug, Clone)]
pub enum FakeRemoteServerStartError {
    #[error("invalid TLS cert or key: {0}")]
    InvalidCertOrKey(Arc<std::io::Error>),
    #[error("server listen error: {0}")]
    ListenError(Arc<std::io::Error>),
    #[error("already started")]
    AlreadyStarted(String),
}

impl PartialEq for FakeRemoteServerStartError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
