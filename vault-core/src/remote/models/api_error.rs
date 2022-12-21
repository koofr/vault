use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ApiError {
    #[serde(rename = "error")]
    pub error: super::ApiErrorDetails,
    #[serde(rename = "requestId")]
    pub request_id: String,
}
