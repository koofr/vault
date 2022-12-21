use serde::{Deserialize, Serialize};

use super::Event;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(tag = "action")]
pub enum Message {
    #[serde(rename = "authenticated")]
    Authenticated,

    #[serde(rename = "registered")]
    Registered {
        #[serde(rename = "requestId")]
        request_id: u32,
        #[serde(rename = "listenerId")]
        listener_id: i64,
    },

    #[serde(rename = "deregistered")]
    Deregistered {
        #[serde(rename = "listenerId")]
        listener_id: i64,
    },

    #[serde(rename = "event")]
    Event {
        #[serde(rename = "listenerId")]
        listener_id: i64,
        event: Event,
    },

    #[serde(other)]
    Unknown,
}
