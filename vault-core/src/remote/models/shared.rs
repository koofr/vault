use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Shared {
    pub files: Vec<super::SharedFile>,
}
