use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemoteName};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Mount {
    pub id: MountId,
    pub name: RemoteName,
    #[serde(rename = "type")]
    pub typ: String,
    pub origin: String,
    pub online: bool,
    #[serde(rename = "isPrimary")]
    pub is_primary: bool,
    #[serde(rename = "spaceTotal", skip_serializing_if = "Option::is_none")]
    pub space_total: Option<i64>,
    #[serde(rename = "spaceUsed", skip_serializing_if = "Option::is_none")]
    pub space_used: Option<i64>,
}
