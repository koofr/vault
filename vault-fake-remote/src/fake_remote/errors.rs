use std::{collections::HashMap, sync::Arc};

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use http::{HeaderName, HeaderValue, StatusCode};
use thiserror::Error;
use uuid::Uuid;
use vault_core::remote::models;

use super::files::objects::object_provider::ObjectProviderError;

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
    ApiError(
        StatusCode,
        ApiErrorCode,
        String,
        Option<HashMap<HeaderName, String>>,
    ),
}

impl From<ObjectProviderError> for FakeRemoteError {
    fn from(value: ObjectProviderError) -> Self {
        Self::ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::Other,
            value.to_string(),
            None,
        )
    }
}

impl IntoResponse for FakeRemoteError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED.into_response(),
            Self::BadRequest(message) => api_error_response(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::BadRequest,
                message,
                None,
            ),
            Self::ApiError(status_code, code, message, headers) => {
                api_error_response(status_code, code, message, headers)
            }
        }
    }
}

fn api_error_response(
    status_code: StatusCode,
    code: ApiErrorCode,
    message: String,
    headers: Option<HashMap<HeaderName, String>>,
) -> Response {
    let mut res = (
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
        .into_response();

    if let Some(headers) = headers {
        let res_headers = res.headers_mut();

        for (name, value) in headers.into_iter() {
            res_headers.insert(name, HeaderValue::try_from(value).unwrap());
        }
    }

    res
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
