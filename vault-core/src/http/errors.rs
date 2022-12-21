use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum HttpError {
    #[error("response error: {0}")]
    ResponseError(String),
}
