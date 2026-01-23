// Copied from
// https://github.com/formatjs/formatjs/blob/f7b5fa0f34331a1b169e5a600a78d8714424a352/crates/icu_messageformat_parser/types.rs,
// removed skeletons, implemented Deserialize

use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, de};

/// Element type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Type {
    /// Raw text
    Literal = 0,
    /// Variable w/o any format, e.g `var` in `this is a {var}`
    Argument = 1,
    /// Variable w/ number format
    Number = 2,
    /// Variable w/ date format
    Date = 3,
    /// Variable w/ time format
    Time = 4,
    /// Variable w/ select format
    Select = 5,
    /// Variable w/ plural format
    Plural = 6,
    /// Only possible within plural argument.
    /// This is the `#` symbol that will be substituted with the count.
    Pound = 7,
    /// XML-like tag
    Tag = 8,
}

impl<'de> Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Type::Literal),
            1 => Ok(Type::Argument),
            2 => Ok(Type::Number),
            3 => Ok(Type::Date),
            4 => Ok(Type::Time),
            5 => Ok(Type::Select),
            6 => Ok(Type::Plural),
            7 => Ok(Type::Pound),
            8 => Ok(Type::Tag),
            _ => Err(serde::de::Error::custom(format!(
                "invalid Type discriminant: {}",
                value
            ))),
        }
    }
}

/// Valid plural rules
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidPluralRule {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
    /// Exact value match (e.g., "=0", "=1")
    Exact(i32),
}

impl<'de> Deserialize<'de> for ValidPluralRule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValidPluralRuleVisitor;

        impl<'de> de::Visitor<'de> for ValidPluralRuleVisitor {
            type Value = ValidPluralRule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    r#"a plural rule ("zero", "one", "two", "few", "many", "other") or an exact rule like "=0""#,
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "zero" => Ok(ValidPluralRule::Zero),
                    "one" => Ok(ValidPluralRule::One),
                    "two" => Ok(ValidPluralRule::Two),
                    "few" => Ok(ValidPluralRule::Few),
                    "many" => Ok(ValidPluralRule::Many),
                    "other" => Ok(ValidPluralRule::Other),
                    _ if v.starts_with('=') => {
                        let number = &v[1..];
                        let parsed = number.parse::<i32>().map_err(|_| {
                            E::custom(format!(
                                "invalid exact plural rule '{}'; expected '=N' where N is an integer",
                                v
                            ))
                        })?;
                        Ok(ValidPluralRule::Exact(parsed))
                    }
                    _ => Err(E::custom(format!("invalid plural rule: {v}"))),
                }
            }
        }

        deserializer.deserialize_str(ValidPluralRuleVisitor)
    }
}

/// Plural type corresponding to Intl.PluralRulesOptions['type']
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluralType {
    Cardinal,
    Ordinal,
}

/// Base element with type and value
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BaseElement {
    pub value: String,
}

/// Literal element
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LiteralElement {
    pub value: String,
}

/// Argument element
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ArgumentElement {
    pub value: String,
}

/// Tag element with children
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TagElement {
    pub value: String,
    pub children: Vec<MessageFormatElement>,
}

/// Simple format element with optional style
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SimpleFormatElement {
    pub value: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub style: Option<S>,
}

/// Number element
pub type NumberElement = SimpleFormatElement;

/// Date element
pub type DateElement = SimpleFormatElement;

/// Time element
pub type TimeElement = SimpleFormatElement;

/// Plural or select option with message elements
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PluralOrSelectOption {
    pub value: Vec<MessageFormatElement>,
}

/// Select element
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SelectElement {
    pub value: String,
    pub options: IndexMap<String, PluralOrSelectOption>,
}

/// Plural element
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PluralElement {
    pub value: String,
    pub options: IndexMap<ValidPluralRule, PluralOrSelectOption>,
    pub offset: i32,
    pub plural_type: PluralType,
}

/// Pound element (#)
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PoundElement {}

/// Message format element (enum of all possible elements)
#[derive(Debug, Clone, PartialEq)]
pub enum MessageFormatElement {
    Literal(LiteralElement),
    Argument(ArgumentElement),
    Number(NumberElement),
    Date(DateElement),
    Time(TimeElement),
    Select(SelectElement),
    Plural(PluralElement),
    Pound(PoundElement),
    Tag(TagElement),
}

// Custom deserialization for MessageFormatElement to match TypeScript format
impl<'de> Deserialize<'de> for MessageFormatElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MessageFormatElementVisitor)
    }
}

struct MessageFormatElementVisitor;

impl<'de> de::Visitor<'de> for MessageFormatElementVisitor {
    type Value = MessageFormatElement;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a MessageFormatElement object")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        use Type::*;

        let mut element_type: Option<Type> = None;

        // Common fields (untyped where necessary)
        let mut value: Option<String> = None;
        // let mut style: Option<String> = None;
        let mut options: Option<serde_json::Value> = None;
        let mut offset: Option<i32> = None;
        let mut plural_type: Option<PluralType> = None;
        let mut children: Option<Vec<MessageFormatElement>> = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "type" => {
                    if element_type.is_some() {
                        return Err(de::Error::duplicate_field("type"));
                    }
                    element_type = Some(map.next_value()?);
                }
                "value" => value = Some(map.next_value()?),
                // "style" => style = Some(map.next_value()?),
                "options" => options = Some(map.next_value()?),
                "offset" => offset = Some(map.next_value()?),
                "pluralType" => plural_type = Some(map.next_value()?),
                "children" => children = Some(map.next_value()?),
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let element_type = element_type.ok_or_else(|| de::Error::missing_field("type"))?;

        match element_type {
            Literal => Ok(MessageFormatElement::Literal(LiteralElement {
                value: value.ok_or_else(|| de::Error::missing_field("value"))?,
            })),

            Argument => Ok(MessageFormatElement::Argument(ArgumentElement {
                value: value.ok_or_else(|| de::Error::missing_field("value"))?,
            })),

            Number => Ok(MessageFormatElement::Number(NumberElement {
                value: value.ok_or_else(|| de::Error::missing_field("value"))?,
                // style: style.ok_or_else(|| de::Error::missing_field("style"))?,
            })),

            Date => Ok(MessageFormatElement::Date(DateElement {
                value: value.ok_or_else(|| de::Error::missing_field("value"))?,
                // style: style.ok_or_else(|| de::Error::missing_field("style"))?,
            })),

            Time => Ok(MessageFormatElement::Time(TimeElement {
                value: value.ok_or_else(|| de::Error::missing_field("value"))?,
                // style: style.ok_or_else(|| de::Error::missing_field("style"))?,
            })),

            Select => {
                let raw = options.ok_or_else(|| de::Error::missing_field("options"))?;

                let options: IndexMap<String, PluralOrSelectOption> =
                    serde_json::from_value(raw).map_err(de::Error::custom)?;

                Ok(MessageFormatElement::Select(SelectElement {
                    value: value.ok_or_else(|| de::Error::missing_field("value"))?,
                    options,
                }))
            }

            Plural => {
                let raw = options.ok_or_else(|| de::Error::missing_field("options"))?;

                let options: IndexMap<ValidPluralRule, PluralOrSelectOption> =
                    serde_json::from_value(raw).map_err(de::Error::custom)?;

                Ok(MessageFormatElement::Plural(PluralElement {
                    value: value.ok_or_else(|| de::Error::missing_field("value"))?,
                    options,
                    offset: offset.ok_or_else(|| de::Error::missing_field("offset"))?,
                    plural_type: plural_type
                        .ok_or_else(|| de::Error::missing_field("pluralType"))?,
                }))
            }

            Pound => Ok(MessageFormatElement::Pound(PoundElement {})),

            Tag => Ok(MessageFormatElement::Tag(TagElement {
                value: value.ok_or_else(|| de::Error::missing_field("value"))?,
                children: children.ok_or_else(|| de::Error::missing_field("children"))?,
            })),
        }
    }
}
