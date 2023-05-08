use thiserror::Error;

use crate::user_error::UserError;

#[derive(Error, Debug, Clone, UserError)]
#[error("{0}")]
pub struct DialogError(String);