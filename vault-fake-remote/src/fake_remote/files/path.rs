use std::str::FromStr;

use serde::de;
use vault_core::{common::errors::InvalidPathError, utils::path_utils};

use super::Name;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path(pub String);

impl Path {
    pub fn root() -> Self {
        Self("/".into())
    }

    pub fn normalize(&self) -> NormalizedPath {
        // lowercase to use for keys and comparisons
        NormalizedPath(self.0.to_lowercase())
    }

    pub fn join_name(&self, name: &Name) -> Self {
        Self(path_utils::join_path_name(&self.0, &name.0))
    }

    pub fn parent(&self) -> Option<Self> {
        path_utils::parent_path(&self.0).map(|path| Self(path.to_owned()))
    }

    pub fn name(&self) -> Option<Name> {
        path_utils::path_to_name(&self.0).map(|name| Name(name.to_owned()))
    }

    pub fn relative_to(&self, path: &Path) -> Option<Self> {
        if path.0 == "/" {
            Some(self.clone())
        } else {
            let self_norm = self.normalize();
            let path_norm = path.normalize();

            if self_norm == path_norm {
                Some(Self("/".into()))
            } else if self_norm.0.starts_with(&format!("{}/", path_norm.0)) {
                Some(Self(self.0[path.0.len()..].to_owned()))
            } else {
                None
            }
        }
    }
}

impl FromStr for Path {
    type Err = InvalidPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // validate and cleanup
        path_utils::normalize_path(s).map(Path)
    }
}

struct FilesystemPathVisitor;

impl<'de> de::Visitor<'de> for FilesystemPathVisitor {
    type Value = Path;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid path")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse().map_err(E::custom)
    }
}

impl<'de> de::Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Path, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(FilesystemPathVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath(pub String);

impl NormalizedPath {
    pub fn root() -> Self {
        Self("/".into())
    }
}

#[cfg(test)]
mod tests {
    use super::Path;

    #[test]
    fn test_path_relative_to() {
        let p = |path: &str| Path(path.to_owned());

        assert_eq!(p("/").relative_to(&p("/")), Some(p("/")));
        assert_eq!(p("/").relative_to(&p("/path")), None);
        assert_eq!(p("/path").relative_to(&p("/path")), Some(p("/")));
        assert_eq!(p("/Path").relative_to(&p("/path")), Some(p("/")));
        assert_eq!(p("/path").relative_to(&p("/Path")), Some(p("/")));
        assert_eq!(p("/pathto").relative_to(&p("/path")), None);
        assert_eq!(p("/path/to").relative_to(&p("/path")), Some(p("/to")));
        assert_eq!(p("/path/to").relative_to(&p("/path")), Some(p("/to")));
        assert_eq!(p("/path/to").relative_to(&p("/Path")), Some(p("/to")));
        assert_eq!(p("/Path/To").relative_to(&p("/path")), Some(p("/To")));
        assert_eq!(
            p("/path/to/file").relative_to(&p("/path")),
            Some(p("/to/file"))
        );
        assert_eq!(
            p("/path/to/file").relative_to(&p("/path/to")),
            Some(p("/file"))
        );
        assert_eq!(p("/path").relative_to(&p("/")), Some(p("/path")));
    }
}
