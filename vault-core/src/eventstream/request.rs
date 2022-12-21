use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "action")]
pub enum Request<'a> {
    #[serde(rename = "auth")]
    Auth { authorization: &'a str },

    #[serde(rename = "register")]
    Register {
        #[serde(rename = "requestId")]
        request_id: u32,
        #[serde(rename = "mountId")]
        mount_id: &'a str,
        #[serde(rename = "path")]
        path: &'a str,
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
