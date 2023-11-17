use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::RemoteName;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesFile {
    pub name: RemoteName,
    #[serde(rename = "type")]
    pub typ: String,
    pub modified: i64,
    pub size: i64,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub tags: HashMap<String, Vec<String>>,
}
