use serde::{Deserialize, Serialize};

use crate::types::RepoId;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Options {
    pub repo_id: RepoId,
}
