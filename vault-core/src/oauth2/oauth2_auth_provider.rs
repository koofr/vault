use std::sync::Arc;

use async_trait::async_trait;

use crate::auth::{errors::AuthError, AuthProvider};

use super::OAuth2Service;

pub struct OAuth2AuthProvider {
    oauth2_service: Arc<OAuth2Service>,
}

impl OAuth2AuthProvider {
    pub fn new(oauth2_service: Arc<OAuth2Service>) -> Self {
        Self { oauth2_service }
    }
}

#[async_trait]
impl AuthProvider for OAuth2AuthProvider {
    async fn get_authorization(&self, force_refresh_token: bool) -> Result<String, AuthError> {
        self.oauth2_service
            .get_authorization(force_refresh_token)
            .await
    }
}
