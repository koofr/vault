use crate::{CatalogFormatter, FormatArguments, FormatError};

pub struct FallbackFormatter {
    default_formatter: Option<CatalogFormatter>,
    current_formatter: Option<CatalogFormatter>,
}

impl FallbackFormatter {
    pub fn new() -> Self {
        Self {
            default_formatter: None,
            current_formatter: None,
        }
    }

    pub fn set_default_formatter(&mut self, formatter: Option<CatalogFormatter>) {
        self.default_formatter = formatter;
    }

    pub fn set_current_formatter(&mut self, formatter: Option<CatalogFormatter>) {
        self.current_formatter = formatter;
    }

    pub fn format_message(
        &self,
        message_id: &str,
        args: FormatArguments<'_>,
    ) -> Result<String, FormatError> {
        if let Some(current_formatter) = &self.current_formatter {
            match current_formatter.format_message(message_id, args) {
                Ok(result) => return Ok(result),
                Err(FormatError::MessageNotFound(..)) => {}
                Err(e) => return Err(e),
            }
        }

        if let Some(default_formatter) = &self.default_formatter {
            return default_formatter.format_message(message_id, args);
        }

        Err(FormatError::MessageNotFound(message_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FormatValue, types::*};
    use std::collections::HashMap;

    fn create_english_message() -> Vec<MessageFormatElement> {
        vec![
            MessageFormatElement::Literal(LiteralElement {
                value: "Do you really want to delete ".to_string(),
            }),
            MessageFormatElement::Plural(PluralElement {
                value: "count".to_string(),
                offset: 0,
                options: [
                    (
                        ValidPluralRule::One,
                        PluralOrSelectOption {
                            value: vec![MessageFormatElement::Literal(LiteralElement {
                                value: "one file".to_string(),
                            })],
                        },
                    ),
                    (
                        ValidPluralRule::Other,
                        PluralOrSelectOption {
                            value: vec![
                                MessageFormatElement::Pound(PoundElement {}),
                                MessageFormatElement::Literal(LiteralElement {
                                    value: " files".to_string(),
                                }),
                            ],
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                plural_type: PluralType::Cardinal,
            }),
            MessageFormatElement::Literal(LiteralElement {
                value: "?".to_string(),
            }),
        ]
    }

    fn create_slovenian_message() -> Vec<MessageFormatElement> {
        vec![
            MessageFormatElement::Literal(LiteralElement {
                value: "Ali res želite odstraniti ".to_string(),
            }),
            MessageFormatElement::Plural(PluralElement {
                value: "count".to_string(),
                offset: 0,
                options: [
                    (
                        ValidPluralRule::One,
                        PluralOrSelectOption {
                            value: vec![
                                MessageFormatElement::Pound(PoundElement {}),
                                MessageFormatElement::Literal(LiteralElement {
                                    value: " datoteko".to_string(),
                                }),
                            ],
                        },
                    ),
                    (
                        ValidPluralRule::Two,
                        PluralOrSelectOption {
                            value: vec![
                                MessageFormatElement::Pound(PoundElement {}),
                                MessageFormatElement::Literal(LiteralElement {
                                    value: " datoteki".to_string(),
                                }),
                            ],
                        },
                    ),
                    (
                        ValidPluralRule::Few,
                        PluralOrSelectOption {
                            value: vec![
                                MessageFormatElement::Pound(PoundElement {}),
                                MessageFormatElement::Literal(LiteralElement {
                                    value: " datoteke".to_string(),
                                }),
                            ],
                        },
                    ),
                    (
                        ValidPluralRule::Other,
                        PluralOrSelectOption {
                            value: vec![
                                MessageFormatElement::Pound(PoundElement {}),
                                MessageFormatElement::Literal(LiteralElement {
                                    value: " datotek".to_string(),
                                }),
                            ],
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                plural_type: PluralType::Cardinal,
            }),
            MessageFormatElement::Literal(LiteralElement {
                value: "?".to_string(),
            }),
        ]
    }

    fn create_english_catalog() -> HashMap<String, Vec<MessageFormatElement>> {
        let mut catalog = HashMap::new();
        catalog.insert("test.message".to_string(), create_english_message());
        catalog
    }

    fn create_slovenian_catalog() -> HashMap<String, Vec<MessageFormatElement>> {
        let mut catalog = HashMap::new();
        catalog.insert("test.message".to_string(), create_slovenian_message());
        catalog
    }

    fn create_empty_catalog() -> HashMap<String, Vec<MessageFormatElement>> {
        HashMap::new()
    }

    #[test]
    fn test_no_default_formatter() {
        let formatter = FallbackFormatter::new();

        assert_eq!(
            formatter.format_message("test.message", &[]),
            Err(FormatError::MessageNotFound("test.message".to_string()))
        );
    }

    #[test]
    fn test_default_formatter() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_english_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter
                .format_message("test.message", &[("count", FormatValue::Integer(1))])
                .unwrap(),
            "Do you really want to delete one file?"
        );
    }

    #[test]
    fn test_default_formatter_missing_message() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_empty_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter.format_message("test.message", &[]),
            Err(FormatError::MessageNotFound("test.message".to_string()))
        );
    }

    #[test]
    fn test_default_formatter_missing_argument() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_english_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter.format_message("test.message", &[]),
            Err(FormatError::MissingArgument("count".to_string()))
        );
    }

    #[test]
    fn test_override_formatter() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_english_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter
                .format_message("test.message", &[("count", FormatValue::Integer(1))])
                .unwrap(),
            "Do you really want to delete one file?"
        );

        formatter.set_current_formatter(Some(
            CatalogFormatter::new(&"sl".parse().unwrap(), create_slovenian_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter
                .format_message("test.message", &[("count", FormatValue::Integer(1))])
                .unwrap(),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_override_formatter_missing_message_in_overridden() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_english_catalog()).unwrap(),
        ));
        formatter.set_current_formatter(Some(
            CatalogFormatter::new(&"sl".parse().unwrap(), create_empty_catalog()).unwrap(),
        ));

        // fallback to default
        assert_eq!(
            formatter
                .format_message("test.message", &[("count", FormatValue::Integer(1))])
                .unwrap(),
            "Do you really want to delete one file?"
        );
    }

    #[test]
    fn test_override_formatter_missing_message_in_both() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_empty_catalog()).unwrap(),
        ));
        formatter.set_current_formatter(Some(
            CatalogFormatter::new(&"sl".parse().unwrap(), create_empty_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter.format_message("test.message", &[]),
            Err(FormatError::MessageNotFound("test.message".to_string()))
        );
    }

    #[test]
    fn test_override_formatter_missing_argument() {
        let mut formatter = FallbackFormatter::new();
        formatter.set_default_formatter(Some(
            CatalogFormatter::new(&"en".parse().unwrap(), create_english_catalog()).unwrap(),
        ));
        formatter.set_current_formatter(Some(
            CatalogFormatter::new(&"sl".parse().unwrap(), create_slovenian_catalog()).unwrap(),
        ));

        assert_eq!(
            formatter.format_message("test.message", &[]),
            Err(FormatError::MissingArgument("count".to_string()))
        );
    }
}
