use thiserror::Error;

use crate::user_error::UserError;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum HttpError {
    #[error("response error: {0}")]
    ResponseError(String),
}

impl UserError for HttpError {
    fn user_error(&self) -> String {
        match self {
            Self::ResponseError(err) => format!("HTTP error: {}", err),
        }
    }
}
