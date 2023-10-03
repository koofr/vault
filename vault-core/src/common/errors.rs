use thiserror::Error;

use crate::user_error::UserError;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("invalid path")]
pub struct InvalidPathError;

impl UserError for InvalidPathError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}
