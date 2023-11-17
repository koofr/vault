use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemotePath};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesCopy {
    #[serde(rename = "toMountId")]
    pub to_mount_id: MountId,
    #[serde(rename = "toPath")]
    pub to_path: RemotePath,
}
