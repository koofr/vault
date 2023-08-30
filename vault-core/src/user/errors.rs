use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("user not found")]
pub struct UserNotFoundError;
