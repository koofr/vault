use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemotePath, RepoId};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VaultRepo {
    pub id: RepoId,
    pub name: String,
    #[serde(rename = "mountId")]
    pub mount_id: MountId,
    pub path: RemotePath,
    pub salt: Option<String>,
    #[serde(rename = "passwordValidator")]
    pub password_validator: String,
    #[serde(rename = "passwordValidatorEncrypted")]
    pub password_validator_encrypted: String,
    pub added: i64,
}
