use serde::{Deserialize, Serialize};

use crate::types::RemoteName;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct FilesFolderCreate {
    pub name: RemoteName,
}
