use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesMove {
    #[serde(rename = "toMountId")]
    pub to_mount_id: String,
    #[serde(rename = "toPath")]
    pub to_path: String,
    #[serde(rename = "ifModified")]
    pub if_modified: Option<i64>,
    #[serde(rename = "ifSize")]
    pub if_size: Option<i64>,
    #[serde(rename = "ifHash")]
    pub if_hash: Option<String>,
}
