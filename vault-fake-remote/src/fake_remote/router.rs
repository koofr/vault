use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use http::{header, Method};
use tower::ServiceBuilder;
use tower_http::cors;

use super::{
    app_state::AppState, eventstream, fix_response_json::fix_response_json, handlers,
    interceptor::interceptor_middleware,
};

pub fn build_router(app_state: AppState) -> Router {
    let cors = cors::CorsLayer::new()
        .allow_headers(vec![
            header::AUTHORIZATION,
            header::CACHE_CONTROL,
            header::CONTENT_TYPE,
            header::IF_MODIFIED_SINCE,
            header::RANGE,
            header::ACCEPT,
            header::ACCEPT_LANGUAGE,
            header::CONTENT_LANGUAGE,
            header::ORIGIN,
            header::REFERER,
        ])
        .allow_methods(vec![
            Method::HEAD,
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_origin(cors::Any)
        .expose_headers(vec![
            header::HeaderName::try_from("X-User-Id").unwrap(),
            header::HeaderName::try_from("X-Request-Id").unwrap(),
            header::HeaderName::try_from("X-File-Info").unwrap(),
        ]);

    Router::new()
        .route("/health", get(handlers::health))
        .route("/oauth2/auth", get(handlers::oauth2_auth))
        .route("/oauth2/token", post(handlers::oauth2_token))
        .route("/api/v2.1/user", get(handlers::user))
        .route(
            "/content/api/v2.1/users/:user_id/profile-picture",
            get(handlers::content_profile_picture),
        )
        .route("/api/v2.1/user/bookmarks", get(handlers::user_bookmarks))
        .route("/api/v2.1/places", get(handlers::places))
        .route("/api/v2.1/shared", get(handlers::shared))
        .route("/api/v2.1/mounts/:mount_id", get(handlers::mounts_details))
        .route("/api/v2.1/mounts/:mount_id/bundle", get(handlers::bundle))
        .route(
            "/api/v2.1/mounts/:mount_id/files/info",
            get(handlers::files_info),
        )
        .route(
            "/api/v2.1/mounts/:mount_id/files/folder",
            post(handlers::files_folder_new),
        )
        .route(
            "/api/v2.1/mounts/:mount_id/files/remove",
            delete(handlers::files_remove),
        )
        .route(
            "/api/v2.1/mounts/:mount_id/files/rename",
            put(handlers::files_rename),
        )
        .route(
            "/api/v2.1/mounts/:mount_id/files/copy",
            put(handlers::files_copy),
        )
        .route(
            "/api/v2.1/mounts/:mount_id/files/move",
            put(handlers::files_move),
        )
        .route(
            "/content/api/v2.1/mounts/:mount_id/files/get",
            get(handlers::content_files_get),
        )
        .route(
            "/content/api/v2.1/mounts/:mount_id/files/listrecursive",
            get(handlers::content_files_list_recursive),
        )
        .route(
            "/content/api/v2.1/mounts/:mount_id/files/put",
            post(handlers::content_files_put),
        )
        .route("/api/v2.1/vault/repos", get(handlers::vault_repos_all))
        .route("/api/v2.1/vault/repos", post(handlers::vault_repos_create))
        .route(
            "/api/v2.1/vault/repos/:repo_id",
            delete(handlers::vault_repos_remove),
        )
        .route("/events", get(eventstream::handler::eventstream))
        .layer(ServiceBuilder::new().layer(cors))
        .layer(middleware::from_fn(fix_response_json))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            interceptor_middleware,
        ))
        .with_state(app_state)
}
