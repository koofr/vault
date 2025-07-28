use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemoteName, RemotePath};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SearchHit {
    #[serde(rename = "mountId")]
    pub mount_id: MountId,
    pub path: RemotePath,
    pub score: f64,
    pub name: RemoteName,
    #[serde(rename = "type")]
    pub typ: String,
    pub modified: i64,
    pub size: i64,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub tags: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub mounts: HashMap<MountId, super::Mount>,
}
