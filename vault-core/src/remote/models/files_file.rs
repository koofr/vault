use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesFile {
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    pub modified: i64,
    pub name: String,
    pub size: i64,
    pub tags: HashMap<String, Vec<String>>,
    #[serde(rename = "type")]
    pub typ: String,
}
