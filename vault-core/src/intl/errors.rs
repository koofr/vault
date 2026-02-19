use thiserror::Error;

use vault_intl::{FormatterError, LanguageIdentifier, LanguageIdentifierParseError};

use crate::{secure_storage::errors::SecureStorageError, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SetLocaleError {
    #[error("failed to parse locale: {0}")]
    LocaleParse(#[from] LanguageIdentifierParseError),
    #[error("locale not found: {0}")]
    LocaleNotFound(LanguageIdentifier),
    #[error("lookup failed: {0:?}")]
    LookupFailed(Vec<LanguageIdentifier>),
    #[error("formatter error: {0}")]
    FormatterError(FormatterError),
    #[error("storage error: {0}")]
    StorageError(#[from] SecureStorageError),
}

impl UserError for SetLocaleError {
    fn user_error(&self) -> String {
        match self {
            Self::LocaleParse(..) => self.to_string(),
            Self::LocaleNotFound(..) => self.to_string(),
            Self::LookupFailed(..) => self.to_string(),
            Self::FormatterError(..) => self.to_string(),
            Self::StorageError(err) => format!("Storage error: {}", err),
        }
    }
}
