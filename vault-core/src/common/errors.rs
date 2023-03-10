use thiserror::Error;

use crate::user_error::UserError;

#[derive(Error, Debug, Clone, PartialEq, UserError)]
#[error("invalid path")]
pub struct InvalidPathError;
