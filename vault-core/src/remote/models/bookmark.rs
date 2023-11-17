use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemoteName, RemotePath};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(rename = "mountId")]
    pub mount_id: MountId,
    pub name: RemoteName,
    pub path: RemotePath,
}
