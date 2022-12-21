use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesMove {
    #[serde(rename = "toMountId")]
    pub to_mount_id: String,
    #[serde(rename = "toPath")]
    pub to_path: String,
}
