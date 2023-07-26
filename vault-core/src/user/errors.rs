use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("user not found")]
pub struct UserNotFoundError;
