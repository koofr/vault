use async_trait::async_trait;

use super::errors::AuthError;

#[async_trait]
pub trait AuthProvider {
    async fn get_authorization(&self, force_refresh_token: bool) -> Result<String, AuthError>;
}
