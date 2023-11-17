use serde::{Deserialize, Serialize};

use crate::types::RemoteName;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SharedFile {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub modified: i64,
    pub mount: super::Mount,
    pub name: RemoteName,
    pub size: i64,
    #[serde(rename = "type")]
    pub typ: String,
}
