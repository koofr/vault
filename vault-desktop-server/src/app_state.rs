use std::sync::Arc;

use vault_web_api::web_vault_base::WebVaultBase;

use crate::{encryption::Encryption, file_handlers::FileHandlers, sessions::Sessions};

#[derive(Clone)]
pub struct AppState {
    pub base: Arc<WebVaultBase>,
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
    pub encryption: Arc<Encryption>,
    pub sessions: Arc<Sessions>,
    pub file_handlers: Arc<FileHandlers>,
}
