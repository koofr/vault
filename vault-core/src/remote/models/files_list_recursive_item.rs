use serde::{Deserialize, Serialize};

use crate::types::RemotePath;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FilesListRecursiveItem {
    #[serde(rename = "file")]
    File {
        path: RemotePath,
        file: super::FilesFile,
    },

    #[serde(rename = "error")]
    Error {
        path: Option<RemotePath>,
        error: super::ApiErrorDetails,
    },
}
