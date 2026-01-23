use std::{collections::HashMap, sync::Arc};

use icu_locale::LanguageIdentifier;
use icu_plurals::PluralRules;

use crate::{
    errors::{CatalogParseError, FormatError, FormatterError},
    format_value::FormatArguments,
    message_formatter::format_message,
    plural_rules::build_plural_rules,
    types::MessageFormatElement,
};

pub struct CatalogFormatter {
    plural_rules: PluralRules,
    catalog: HashMap<String, Vec<MessageFormatElement>>,
}

impl CatalogFormatter {
    pub fn new(
        locale: &LanguageIdentifier,
        catalog: HashMap<String, Vec<MessageFormatElement>>,
    ) -> Result<Self, FormatterError> {
        let plural_rules = build_plural_rules(locale).map_err(FormatterError::ICU4X)?;

        Ok(Self {
            plural_rules,
            catalog,
        })
    }

    pub fn from_str(locale: &LanguageIdentifier, catalog: &str) -> Result<Self, FormatterError> {
        let catalog = serde_json::from_str(catalog)
            .map_err(|err| FormatterError::CatalogParse(CatalogParseError(Arc::new(err))))?;

        Self::new(locale, catalog)
    }

    pub fn format_message(
        &self,
        message_id: &str,
        args: FormatArguments<'_>,
    ) -> Result<String, FormatError> {
        match self.catalog.get(message_id) {
            Some(message) => {
                let mut s = String::new();
                format_message(message, args, &self.plural_rules, &mut s)?;
                Ok(s)
            }
            None => Err(FormatError::MessageNotFound(message_id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FormatValue, types::*};

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

    #[test]
    fn test_format_message() {
        let mut catalog = HashMap::new();
        catalog.insert("test.message".to_string(), create_english_message());
        let formatter = CatalogFormatter::new(&"en".parse().unwrap(), catalog).unwrap();

        let result = formatter
            .format_message("test.message", &[("count", FormatValue::Integer(1))])
            .unwrap();
        assert_eq!(result, "Do you really want to delete one file?");
    }

    #[test]
    fn test_format_message_message_not_found() {
        let catalog = HashMap::new();
        let formatter = CatalogFormatter::new(&"en".parse().unwrap(), catalog).unwrap();

        let result = formatter.format_message("nonexistent.message", &[]);
        assert_eq!(
            result,
            Err(FormatError::MessageNotFound(
                "nonexistent.message".to_string()
            ))
        );
    }
}
