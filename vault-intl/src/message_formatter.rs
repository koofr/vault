use icu_plurals::{PluralCategory, PluralRules};

use crate::{
    errors::FormatError,
    format_value::{FormatArguments, FormatValue},
    types,
};

#[derive(Clone)]
struct PluralContext {
    effective_value: i32,
}

pub fn format_message<W: std::fmt::Write>(
    message: &[types::MessageFormatElement],
    args: FormatArguments<'_>,
    plural_rules: &PluralRules,
    f: &mut W,
) -> Result<(), FormatError> {
    let mut plural_context_stack = Vec::new();

    format_message_with_context(message, args, plural_rules, &mut plural_context_stack, f)
}

fn format_message_with_context<W: std::fmt::Write>(
    message: &[types::MessageFormatElement],
    args: FormatArguments<'_>,
    plural_rules: &PluralRules,
    plural_context_stack: &mut Vec<PluralContext>,
    f: &mut W,
) -> Result<(), FormatError> {
    for element in message {
        match element {
            types::MessageFormatElement::Literal(types::LiteralElement { value, .. }) => {
                f.write_str(value)?;
            }
            types::MessageFormatElement::Argument(types::ArgumentElement { value, .. }) => {
                let arg_value = args
                    .iter()
                    .find(|(k, _)| *k == value)
                    .map(|(_, v)| v)
                    .ok_or_else(|| FormatError::MissingArgument(value.to_string()))?;

                match arg_value {
                    FormatValue::String(s) => f.write_str(s)?,
                    FormatValue::Integer(i) => write!(f, "{}", i)?,
                }
            }
            // {gender, select,
            //   male {He will respond shortly.}
            //   female {She will respond shortly.}
            //   other {They will respond shortly.}
            // }
            types::MessageFormatElement::Select(types::SelectElement {
                value, options, ..
            }) => {
                let arg_value = args
                    .iter()
                    .find(|(k, _)| *k == value)
                    .map(|(_, v)| v)
                    .ok_or_else(|| FormatError::MissingArgument(value.to_string()))?;

                let select_key = match arg_value {
                    FormatValue::String(s) => *s,
                    FormatValue::Integer(_) => {
                        return Err(FormatError::InvalidArgumentType(value.clone()));
                    }
                };

                let selected_option = options
                    .get(select_key)
                    .or_else(|| options.get("other"))
                    .ok_or(FormatError::MissingSelectOther)?;

                format_message_with_context(
                    &selected_option.value,
                    args,
                    plural_rules,
                    plural_context_stack,
                    f,
                )?;
            }
            // {itemCount, plural,
            //   =0 {You have no items.}
            //   one {You have {itemCount, number} item.}
            //   other {You have {itemCount, number} items.}
            // }
            types::MessageFormatElement::Plural(types::PluralElement {
                value,
                plural_type,
                offset,
                options,
                ..
            }) => {
                // Only Cardinal plural types are supported for now
                if *plural_type != types::PluralType::Cardinal {
                    return Err(FormatError::UnsupportedFeature("Non-cardinal plural types"));
                }

                let arg_value = args
                    .iter()
                    .find(|(k, _)| *k == value)
                    .map(|(_, v)| v)
                    .ok_or_else(|| FormatError::MissingArgument(value.to_string()))?;

                let count = match arg_value {
                    FormatValue::String(_) => {
                        return Err(FormatError::InvalidArgumentType(value.clone()));
                    }
                    FormatValue::Integer(i) => *i,
                };

                let effective_value = (count as i32) - *offset;

                // First, check for exact matches
                let mut selected_option = None;
                for (key, option) in options {
                    if let types::ValidPluralRule::Exact(exact_value) = key {
                        if *exact_value == effective_value {
                            selected_option = Some(option);
                            break;
                        }
                    }
                }

                // If no exact match, use ICU plural rules
                if selected_option.is_none() {
                    let category = plural_rules.category_for(effective_value);

                    // Try to find a matching category key
                    for (key, option) in options {
                        match (key, category) {
                            (types::ValidPluralRule::Zero, PluralCategory::Zero) => {
                                selected_option = Some(option);
                                break;
                            }
                            (types::ValidPluralRule::One, PluralCategory::One) => {
                                selected_option = Some(option);
                                break;
                            }
                            (types::ValidPluralRule::Two, PluralCategory::Two) => {
                                selected_option = Some(option);
                                break;
                            }
                            (types::ValidPluralRule::Few, PluralCategory::Few) => {
                                selected_option = Some(option);
                                break;
                            }
                            (types::ValidPluralRule::Many, PluralCategory::Many) => {
                                selected_option = Some(option);
                                break;
                            }
                            (types::ValidPluralRule::Other, PluralCategory::Other) => {
                                selected_option = Some(option);
                                break;
                            }
                            _ => continue,
                        }
                    }
                }

                let selected_option = selected_option.ok_or(FormatError::MissingPluralOther)?;

                plural_context_stack.push(PluralContext { effective_value });

                let res = format_message_with_context(
                    &selected_option.value,
                    args,
                    plural_rules,
                    plural_context_stack,
                    f,
                );

                plural_context_stack.pop();

                res?;
            }
            // This is the `#` symbol that will be substituted with the count.
            types::MessageFormatElement::Pound(_) => {
                let context = plural_context_stack
                    .last()
                    .ok_or(FormatError::InvalidPoundUsage)?;

                write!(f, "{}", context.effective_value)?;
            }
            types::MessageFormatElement::Number(_) => {
                return Err(FormatError::UnsupportedFeature("NumberElement"));
            }
            types::MessageFormatElement::Date(_) => {
                return Err(FormatError::UnsupportedFeature("DateElement"));
            }
            types::MessageFormatElement::Time(_) => {
                return Err(FormatError::UnsupportedFeature("TimeElement"));
            }
            types::MessageFormatElement::Tag(_) => {
                return Err(FormatError::UnsupportedFeature("TagElement"));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::plural_rules::build_plural_rules;

    use super::*;

    fn create_english_message() -> Vec<types::MessageFormatElement> {
        vec![
            types::MessageFormatElement::Literal(types::LiteralElement {
                value: "Do you really want to delete ".to_string(),
            }),
            types::MessageFormatElement::Plural(types::PluralElement {
                value: "count".to_string(),
                offset: 0,
                options: [
                    (
                        types::ValidPluralRule::One,
                        types::PluralOrSelectOption {
                            value: vec![types::MessageFormatElement::Literal(
                                types::LiteralElement {
                                    value: "one file".to_string(),
                                },
                            )],
                        },
                    ),
                    (
                        types::ValidPluralRule::Other,
                        types::PluralOrSelectOption {
                            value: vec![
                                types::MessageFormatElement::Pound(types::PoundElement {}),
                                types::MessageFormatElement::Literal(types::LiteralElement {
                                    value: " files".to_string(),
                                }),
                            ],
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                plural_type: types::PluralType::Cardinal,
            }),
            types::MessageFormatElement::Literal(types::LiteralElement {
                value: "?".to_string(),
            }),
        ]
    }

    fn create_slovenian_message() -> Vec<types::MessageFormatElement> {
        vec![
            types::MessageFormatElement::Literal(types::LiteralElement {
                value: "Ali res želite odstraniti ".to_string(),
            }),
            types::MessageFormatElement::Plural(types::PluralElement {
                value: "count".to_string(),
                offset: 0,
                options: [
                    (
                        types::ValidPluralRule::One,
                        types::PluralOrSelectOption {
                            value: vec![
                                types::MessageFormatElement::Pound(types::PoundElement {}),
                                types::MessageFormatElement::Literal(types::LiteralElement {
                                    value: " datoteko".to_string(),
                                }),
                            ],
                        },
                    ),
                    (
                        types::ValidPluralRule::Two,
                        types::PluralOrSelectOption {
                            value: vec![
                                types::MessageFormatElement::Pound(types::PoundElement {}),
                                types::MessageFormatElement::Literal(types::LiteralElement {
                                    value: " datoteki".to_string(),
                                }),
                            ],
                        },
                    ),
                    (
                        types::ValidPluralRule::Few,
                        types::PluralOrSelectOption {
                            value: vec![
                                types::MessageFormatElement::Pound(types::PoundElement {}),
                                types::MessageFormatElement::Literal(types::LiteralElement {
                                    value: " datoteke".to_string(),
                                }),
                            ],
                        },
                    ),
                    (
                        types::ValidPluralRule::Other,
                        types::PluralOrSelectOption {
                            value: vec![
                                types::MessageFormatElement::Pound(types::PoundElement {}),
                                types::MessageFormatElement::Literal(types::LiteralElement {
                                    value: " datotek".to_string(),
                                }),
                            ],
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                plural_type: types::PluralType::Cardinal,
            }),
            types::MessageFormatElement::Literal(types::LiteralElement {
                value: "?".to_string(),
            }),
        ]
    }

    #[test]
    fn test_arguments() {
        let mut result = String::new();
        format_message(
            &[
                types::MessageFormatElement::Literal(types::LiteralElement {
                    value: "Hello ".to_string(),
                }),
                types::MessageFormatElement::Argument(types::ArgumentElement {
                    value: "name".to_string(),
                }),
            ],
            &[("name", FormatValue::String("world"))],
            &build_plural_rules(&"en".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_english_plural() {
        // Test singular
        let mut result = String::new();
        format_message(
            &create_english_message(),
            &[("count", FormatValue::Integer(1))],
            &build_plural_rules(&"en".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        assert_eq!(result, "Do you really want to delete one file?");

        // Test plural
        let mut result = String::new();
        format_message(
            &create_english_message(),
            &[("count", FormatValue::Integer(3))],
            &build_plural_rules(&"en".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        assert_eq!(result, "Do you really want to delete 3 files?");
    }

    #[test]
    fn test_slovenian_plural() {
        // Test singular (one)
        let mut result = String::new();
        format_message(
            &create_slovenian_message(),
            &[("count", FormatValue::Integer(1))],
            &build_plural_rules(&"sl".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        // incorrect because of missing plural rules for sl
        assert_eq!(result, "Ali res želite odstraniti 1 datotek?");

        // Test two
        let mut result = String::new();
        format_message(
            &create_slovenian_message(),
            &[("count", FormatValue::Integer(2))],
            &build_plural_rules(&"sl".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        // incorrect because of missing plural rules for sl
        assert_eq!(result, "Ali res želite odstraniti 2 datotek?");

        // Test few
        let mut result = String::new();
        format_message(
            &create_slovenian_message(),
            &[("count", FormatValue::Integer(3))],
            &build_plural_rules(&"sl".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        // incorrect because of missing plural rules for sl
        assert_eq!(result, "Ali res želite odstraniti 3 datotek?");

        // Test other
        let mut result = String::new();
        format_message(
            &create_slovenian_message(),
            &[("count", FormatValue::Integer(5))],
            &build_plural_rules(&"sl".parse().unwrap()).unwrap(),
            &mut result,
        )
        .unwrap();
        assert_eq!(result, "Ali res želite odstraniti 5 datotek?");
    }

    #[test]
    fn test_missing_argument() {
        let mut s = String::new();
        let result = format_message(
            &[types::MessageFormatElement::Argument(
                types::ArgumentElement {
                    value: "missing".to_string(),
                },
            )],
            &[],
            &build_plural_rules(&"en".parse().unwrap()).unwrap(),
            &mut s,
        );
        assert_eq!(
            result,
            Err(FormatError::MissingArgument("missing".to_string()))
        );
    }
}
