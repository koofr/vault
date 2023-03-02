use async_trait::async_trait;

use super::{errors::AuthError, AuthProvider};

pub struct MockAuthProvider {
    on_get_authorization: Box<dyn Fn(bool) -> Result<String, AuthError> + Send + Sync>,
}

impl MockAuthProvider {
    pub fn new(
        on_get_authorization: Box<dyn Fn(bool) -> Result<String, AuthError> + Send + Sync>,
    ) -> Self {
        Self {
            on_get_authorization,
        }
    }

    pub fn default() -> Self {
        Self {
            on_get_authorization: Box::new(|_| Ok(String::from("Bearer TESTTOKEN"))),
        }
    }
}

#[async_trait]
impl AuthProvider for MockAuthProvider {
    async fn get_authorization(&self, force_refresh_token: bool) -> Result<String, AuthError> {
        (self.on_get_authorization)(force_refresh_token)
    }
}
