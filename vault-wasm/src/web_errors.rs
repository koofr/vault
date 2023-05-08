use std::sync::Arc;

use vault_core::{user_error::UserError, Vault};

pub struct WebErrors {
    vault: Arc<Vault>,
}

impl WebErrors {
    pub fn new(vault: Arc<Vault>) -> Self {
        Self { vault }
    }

    pub fn handle_error_str(&self, error_str: String) {
        self.vault.notifications_show(error_str);
    }

    pub fn handle_error(&self, user_error: impl UserError) {
        self.handle_error_str(user_error.user_error());
    }

    pub fn handle_result(&self, result: Result<(), impl UserError>) {
        match result {
            Ok(()) => (),
            Err(err) => self.handle_error(err),
        }
    }
}
