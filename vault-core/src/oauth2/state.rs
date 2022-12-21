use serde::{Deserialize, Serialize};

use crate::common::state::Status;

use super::errors::OAuth2Error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: f64,
}

#[derive(Clone, Default)]
pub struct OAuth2State {
    pub status: Status<OAuth2Error>,
    pub token: Option<OAuth2Token>,
}
