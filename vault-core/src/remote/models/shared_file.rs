use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SharedFile {
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub modified: i64,
    pub mount: super::Mount,
    pub name: String,
    pub size: i64,
    #[serde(rename = "type")]
    pub typ: String,
}
