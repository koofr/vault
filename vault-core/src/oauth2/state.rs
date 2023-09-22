use serde::{Deserialize, Serialize};

use crate::common::state::Status;

use super::errors::OAuth2Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: f64,
}

#[derive(Debug, Clone)]
pub enum FinishFlowResult {
    LoggedIn,
    LoggedOut,
}

#[derive(Debug, Clone)]
pub struct OAuth2State {
    pub status: Status<OAuth2Error>,
    pub token: Option<OAuth2Token>,
}

impl Default for OAuth2State {
    fn default() -> Self {
        Self {
            status: Status::Loading { loaded: false },
            token: None,
        }
    }
}
