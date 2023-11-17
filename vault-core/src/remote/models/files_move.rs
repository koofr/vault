use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemotePath};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesMove {
    #[serde(rename = "toMountId")]
    pub to_mount_id: MountId,
    #[serde(rename = "toPath")]
    pub to_path: RemotePath,
    #[serde(rename = "ifModified")]
    pub if_modified: Option<i64>,
    #[serde(rename = "ifSize")]
    pub if_size: Option<i64>,
    #[serde(rename = "ifHash")]
    pub if_hash: Option<String>,
}
