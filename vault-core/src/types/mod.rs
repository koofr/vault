use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

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
