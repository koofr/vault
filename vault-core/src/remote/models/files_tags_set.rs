use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesTagsSet {
    #[serde(rename = "tags")]
    pub tags: HashMap<String, Vec<String>>,
    #[serde(rename = "ifModified", skip_serializing_if = "Option::is_none")]
    pub if_modified: Option<i64>,
    #[serde(rename = "ifSize", skip_serializing_if = "Option::is_none")]
    pub if_size: Option<i64>,
    #[serde(rename = "ifHash", skip_serializing_if = "Option::is_none")]
    pub if_hash: Option<String>,
    #[serde(rename = "ifOldTags", skip_serializing_if = "Option::is_none")]
    pub if_old_tags: Option<HashMap<String, Vec<String>>>,
}
