use std::str::FromStr;

use serde::de;
use vault_core::{common::errors::InvalidPathError, utils::name_utils};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(pub String);

impl Name {
    pub fn ext(&self) -> Option<String> {
        name_utils::name_to_ext(&self.0.to_lowercase()).map(str::to_string)
    }
}

impl FromStr for Name {
    type Err = InvalidPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // validate
        name_utils::validate_name(s)?;

        Ok(Self(s.to_owned()))
    }
}

struct FilesystemPathVisitor;

impl<'de> de::Visitor<'de> for FilesystemPathVisitor {
    type Value = Name;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(E::custom)
    }
}

impl<'de> de::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Name, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(FilesystemPathVisitor)
    }
}
