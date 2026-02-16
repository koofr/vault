use thiserror::Error;

use crate::{intl, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum HttpError {
    #[error("response error: {0}")]
    ResponseError(String),
}

impl UserError for HttpError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        match self {
            Self::ResponseError(err) => intl::format_message!(
                intl_service,
                "core.http.http_response.error",
                "Error shown when an HTTP API response fails (usually for unexpected reasons).",
                "HTTP error: {error}",
                &[("error", intl::FormatValue::String(err))]
            ),
        }
    }
}
