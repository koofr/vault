use thiserror::Error;

use crate::user_error::UserError;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct DialogError(String);

impl UserError for DialogError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}
