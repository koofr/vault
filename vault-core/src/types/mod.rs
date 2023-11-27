use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct MountId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RemotePath(pub String);

impl RemotePath {
    pub fn is_root(&self) -> bool {
        self.0 == "/"
    }

    pub fn to_lowercase(&self) -> RemotePathLower {
        RemotePathLower(self.0.to_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RemotePathLower(pub String);

impl RemotePathLower {
    pub fn is_root(&self) -> bool {
        self.0 == "/"
    }
}

lazy_static! {
    pub static ref REMOTE_PATH_LOWER_ROOT: RemotePathLower = RemotePathLower("/".into());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RemoteName(pub String);

impl RemoteName {
    pub fn to_lowercase(&self) -> RemoteNameLower {
        RemoteNameLower(self.0.to_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RemoteNameLower(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RemoteFileId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RepoId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct EncryptedPath(pub String);

impl EncryptedPath {
    pub fn is_root(&self) -> bool {
        self.0 == "/"
    }
}

lazy_static! {
    pub static ref ENCRYPTED_PATH_ROOT: EncryptedPath = EncryptedPath("/".into());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct DecryptedPath(pub String);

impl DecryptedPath {
    pub fn is_root(&self) -> bool {
        self.0 == "/"
    }
}

lazy_static! {
    pub static ref DECRYPTED_PATH_ROOT: DecryptedPath = DecryptedPath("/".into());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct EncryptedName(pub String);

impl EncryptedName {
    pub fn to_lowercase(&self) -> EncryptedNameLower {
        EncryptedNameLower(self.0.to_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct EncryptedNameLower(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct DecryptedName(pub String);

impl DecryptedName {
    pub fn to_lowercase(&self) -> DecryptedNameLower {
        DecryptedNameLower(self.0.to_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct DecryptedNameLower(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct RepoFileId(pub String);

/// Unix timestamp in milliseconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TimeMillis(pub i64);

impl std::ops::Sub<TimeMillis> for TimeMillis {
    // chrono::Duration instead of std::time::Duration because
    // std::time::Duration does not support negative values
    type Output = chrono::Duration;

    fn sub(self, rhs: TimeMillis) -> Self::Output {
        chrono::Duration::milliseconds(self.0 - rhs.0)
    }
}

impl std::ops::Add<std::time::Duration> for TimeMillis {
    type Output = TimeMillis;

    fn add(self, rhs: std::time::Duration) -> Self::Output {
        TimeMillis(self.0 + rhs.as_millis() as i64)
    }
}

impl std::ops::Sub<std::time::Duration> for TimeMillis {
    type Output = TimeMillis;

    fn sub(self, rhs: std::time::Duration) -> Self::Output {
        TimeMillis(self.0 - rhs.as_millis() as i64)
    }
}

impl std::ops::Add<chrono::Duration> for TimeMillis {
    type Output = TimeMillis;

    fn add(self, rhs: chrono::Duration) -> Self::Output {
        TimeMillis(self.0 + rhs.num_milliseconds() as i64)
    }
}

impl std::ops::Sub<chrono::Duration> for TimeMillis {
    type Output = TimeMillis;

    fn sub(self, rhs: chrono::Duration) -> Self::Output {
        TimeMillis(self.0 - rhs.num_milliseconds() as i64)
    }
}

impl Serialize for TimeMillis {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // f64 for json compatibility
        serializer.serialize_f64(self.0 as f64)
    }
}

impl<'de> Deserialize<'de> for TimeMillis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // f64 for json compatibility
        let value: f64 = Deserialize::deserialize(deserializer)?;

        Ok(TimeMillis(value as i64))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::types::MountId;

    #[test]
    fn test_serde_json() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Mount {
            pub id: MountId,
            pub name: String,
        }

        assert_eq!(
            serde_json::to_string(&Mount {
                id: MountId("MOUNTID".into()),
                name: "Name".into(),
            })
            .unwrap(),
            r#"{"id":"MOUNTID","name":"Name"}"#,
        );

        assert_eq!(
            serde_json::from_str::<Mount>(r#"{"id":"MOUNTID","name":"Name"}"#).unwrap(),
            Mount {
                id: MountId("MOUNTID".into()),
                name: "Name".into(),
            },
        );
    }
}
