use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FilesListRecursiveItem {
    #[serde(rename = "file")]
    File { path: String, file: super::FilesFile },

    #[serde(rename = "error")]
    Error { path: Option<String>, error: super::ApiErrorDetails },
}
