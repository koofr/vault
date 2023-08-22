use std::sync::Arc;

use vault_core::{oauth2::OAuth2Config, secure_storage::SecureStorage, Vault};

use crate::{
    native_eventstream_websocket_client::{
        get_tokio_tungstenite_connector, NativeEventstreamWebSocketClient,
    },
    native_http_client::{get_reqwest_client, NativeHttpClient},
    native_runtime::NativeRuntime,
};

pub fn accept_invalid_certs(base_url: &str) -> bool {
    base_url.starts_with("https://127.0.0.1:") || base_url.starts_with("https://localhost:")
}

pub fn build_vault(
    base_url: String,
    oauth2_config: OAuth2Config,
    secure_storage: Box<dyn SecureStorage + Send + Sync>,
    tokio_runtime: Arc<tokio::runtime::Runtime>,
) -> (
    Arc<Vault>,
    Arc<reqwest::Client>,
    Option<tokio_tungstenite::Connector>,
) {
    let accept_invalid_certs = accept_invalid_certs(&base_url);

    let reqwest_client = Arc::new(get_reqwest_client(accept_invalid_certs));
    let http_client = Box::new(NativeHttpClient::new(reqwest_client.clone()));

    let tokio_tungstenite_connector = get_tokio_tungstenite_connector(accept_invalid_certs);
    let eventstream_websocket_client = Box::new(NativeEventstreamWebSocketClient::new(
        tokio_runtime.clone(),
        tokio_tungstenite_connector.clone(),
    ));

    let runtime = Box::new(NativeRuntime::new(tokio_runtime.clone()));

    let vault = Arc::new(Vault::new(
        base_url.clone(),
        oauth2_config,
        http_client,
        eventstream_websocket_client,
        secure_storage,
        runtime,
    ));

    (vault, reqwest_client, tokio_tungstenite_connector)
}
