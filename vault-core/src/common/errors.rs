use thiserror::Error;

use crate::{intl, user_error::UserError};

#[derive(Error, Debug, Clone, PartialEq)]
#[error("invalid path")]
pub struct InvalidPathError;

impl UserError for InvalidPathError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.common.invalid_path.error",
            "Validation error shown when the provided path is invalid.",
            "Path is not valid"
        )
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("invalid name: {escaped_name}")]
pub struct InvalidNameError {
    pub name: String,
    pub escaped_name: String,
}

impl InvalidNameError {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            escaped_name: Self::escape_name(name),
        }
    }

    pub fn escape_name(name: &str) -> String {
        String::from_utf8(
            name.bytes()
                .flat_map(|b| std::ascii::escape_default(b))
                .collect::<Vec<u8>>(),
        )
        .unwrap()
    }
}

impl UserError for InvalidNameError {
    fn user_error(&self, intl_service: &intl::IntlService) -> String {
        intl::format_message!(
            intl_service,
            "core.common.invalid_name.error",
            "Validation error shown when a file or folder name is invalid.",
            "Name is not valid"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::InvalidNameError;

    #[test]
    pub fn test_invalid_name_error() {
        assert_eq!(
            InvalidNameError::new("Hello world\0").to_string(),
            "invalid name: Hello world\\x00"
        )
    }
}
