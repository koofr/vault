use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum FormatError {
    #[error("missing argument: {0}")]
    MissingArgument(String),
    #[error("invalid argument type: {0}")]
    InvalidArgumentType(String),
    #[error("missing plural 'other' case")]
    MissingPluralOther,
    #[error("missing select 'other' case")]
    MissingSelectOther,
    #[error("invalid pound usage outside plural context")]
    InvalidPoundUsage,
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(&'static str),
    #[error("message not found: {0}")]
    MessageNotFound(String),
    #[error("failed to format")]
    Format(#[from] std::fmt::Error),
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum FormatterError {
    #[error("failed to parse locale: {0}")]
    ICU4X(icu_provider::DataError),
    #[error("{0}")]
    CatalogParse(CatalogParseError),
}

#[derive(Error, Debug, Clone)]
#[error("failed to parse catalog: {0}")]
pub struct CatalogParseError(pub Arc<serde_json::Error>);

impl PartialEq for CatalogParseError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}
