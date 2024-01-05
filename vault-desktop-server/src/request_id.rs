use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestId {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(
        default,
        rename = "sequenceId",
        skip_serializing_if = "Option::is_none"
    )]
    pub sequence_id: Option<u64>,
}
