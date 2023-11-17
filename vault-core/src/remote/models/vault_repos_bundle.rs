use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::MountId;

use super::{Mount, VaultRepo};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct VaultReposBundle {
    pub repos: Vec<VaultRepo>,
    pub mounts: HashMap<MountId, Mount>,
}
