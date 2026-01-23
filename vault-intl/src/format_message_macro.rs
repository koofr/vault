#[macro_export]
macro_rules! format_message {
    ($formatter:expr, $message_id:expr, $description:expr, $default_message:expr, $args:expr) => {
        $formatter.format_message($message_id, $args)
    };
    ($formatter:expr, $message_id:expr, $description:expr, $default_message:expr) => {
        $formatter.format_message($message_id, &[])
    };
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, vec};

    use crate::{CatalogFormatter, FormatValue, types::*};

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
    fn test_format_message_macro() {
        let mut catalog = HashMap::new();
        catalog.insert("test.message".to_string(), create_english_message());
        let formatter = CatalogFormatter::new(&"en".parse().unwrap(), catalog).unwrap();

        let result = format_message!(
            formatter,
            "test.message",
            "Message displayed in delete file dialog",
            "Do you really want to delete {count, plural, one {one file} other {# files}}?",
            &[("count", FormatValue::Integer(1))]
        )
        .unwrap();
        assert_eq!(result, "Do you really want to delete one file?");
    }

    #[test]
    fn test_format_message_macro_default_args() {
        let mut catalog = HashMap::new();
        catalog.insert(
            "test.message".to_string(),
            vec![MessageFormatElement::Literal(LiteralElement {
                value: "Hello".to_string(),
            })],
        );
        let formatter = CatalogFormatter::new(&"en".parse().unwrap(), catalog).unwrap();

        let result = format_message!(formatter, "test.message", "A test message", "Hello").unwrap();
        assert_eq!(result, "Hello");
    }
}
