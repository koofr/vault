use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum HttpError {
    #[error("response error: {0}")]
    ResponseError(String),
}
