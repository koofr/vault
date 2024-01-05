use std::{net::SocketAddr, sync::Arc};

use axum::{
    http::{header::CONTENT_TYPE, Method},
    Router, ServiceExt,
};
use tower::Layer;
use tower_http::cors::{Any, CorsLayer};
use vault_web_api::web_vault_base::WebVaultBase;

use crate::{
    app_state::AppState,
    encryption::Encryption,
    file_handlers::FileHandlers,
    handlers::register_routes,
    request_encryption::{encryption_middleware, EncryptionMiddlewareState},
    sessions::Sessions,
};

pub async fn app(
    port: u16,
    web_vault: WebVaultBase,
    tokio_runtime: Arc<tokio::runtime::Runtime>,
    encryption: Arc<Encryption>,
    file_handlers: Arc<FileHandlers>,
) {
    let sessions = Arc::new(Sessions::new());

    let app_state = AppState {
        base: Arc::new(web_vault),
        tokio_runtime,
        encryption,
        sessions,
        file_handlers,
    };

    let app = register_routes(Router::new())
        .with_state(app_state.clone())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_headers(vec![CONTENT_TYPE]),
        );

    let app = axum::middleware::from_fn_with_state(
        EncryptionMiddlewareState {
            encryption: app_state.encryption.clone(),
            sessions: app_state.sessions.clone(),
        },
        encryption_middleware,
    )
    .layer(app);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Backend is listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
