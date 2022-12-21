use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Bookmark {
    #[serde(rename = "mountId")]
    pub mount_id: String,
    pub name: String,
    pub path: String,
}
