use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    #[error("response error: {0}")]
    ResponseError(String),
}
