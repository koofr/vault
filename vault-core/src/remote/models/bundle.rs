use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Bundle {
    pub file: super::BundleFile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<super::BundleFile>>,
}
