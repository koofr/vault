use serde::{Deserialize, Serialize};

use crate::types::{MountId, RemotePath};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Request {
    #[serde(rename = "auth")]
    Auth { authorization: String },

    #[serde(rename = "register")]
    Register {
        #[serde(rename = "requestId")]
        request_id: Option<u32>,
        #[serde(rename = "mountId")]
        mount_id: MountId,
        #[serde(rename = "path")]
        path: RemotePath,
    },

    #[serde(rename = "deregister")]
    Deregister {
        #[serde(rename = "listenerId")]
        listener_id: i64,
    },

    #[serde(rename = "ping")]
    Ping,

    #[serde(other)]
    Unknown,
}
