use std::{net::SocketAddr, sync::Arc};

use futures::FutureExt;

use super::{
    app_state::AppState,
    context::Context,
    debug_interceptor::get_debug_interceptor,
    errors::FakeRemoteServerStartError,
    files::objects::object_provider::BoxObjectProvider,
    router::build_router,
    server::{FakeRemoteServer, FakeRemoteServerListener},
    CERT_PEM, KEY_PEM,
};

#[derive(Debug, Clone)]
pub struct FakeRemoteAppConfig {
    pub http_addr: SocketAddr,
    pub https_addr: SocketAddr,
    pub object_provider: Arc<BoxObjectProvider>,
    pub user_id: String,
    pub mount_id: String,
    pub oauth2_access_token: String,
    pub oauth2_refresh_token: String,
    pub create_vault_repo: bool,
}

pub struct FakeRemoteApp {
    app_state: AppState,
    http_server: FakeRemoteServer,
    https_server: FakeRemoteServer,
}

impl FakeRemoteApp {
    pub async fn new(
        config: FakeRemoteAppConfig,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    ) -> Self {
        let mut app_state = AppState::new(config.object_provider.clone());

        init_state(&app_state, config.clone()).await;

        let reset_app_state = app_state.clone();
        let reset_config = config.clone();
        let reset = Box::new(move || {
            let app_state = reset_app_state.clone();
            let config = reset_config.clone();

            async move { reset(app_state, config).await }.boxed()
        });

        app_state.interceptor = Arc::new(Some(get_debug_interceptor(
            Default::default(),
            app_state.clone(),
            reset,
        )));

        let http_server = FakeRemoteServer::new(
            FakeRemoteServerListener::Http {
                proposed_addr: config.http_addr,
            },
            tokio_runtime.clone(),
        );

        let https_server = FakeRemoteServer::new(
            FakeRemoteServerListener::Https {
                proposed_addr: config.https_addr.clone(),
                cert_pem: CERT_PEM.to_owned(),
                key_pem: KEY_PEM.to_owned(),
            },
            tokio_runtime.clone(),
        );

        Self {
            app_state,
            http_server,
            https_server,
        }
    }

    pub async fn start(&self) -> Result<(String, String), FakeRemoteServerStartError> {
        let http_url = self
            .http_server
            .start(build_router(self.app_state.clone()))
            .await?;

        let https_url = self
            .https_server
            .start(build_router(self.app_state.clone()))
            .await?;

        Ok((http_url, https_url))
    }

    pub async fn stop(&self) {
        self.http_server.stop().await;

        self.https_server.stop().await;
    }
}

async fn init_state(
    AppState {
        state,
        users_service,
        vault_repos_create_service,
        ..
    }: &AppState,
    FakeRemoteAppConfig {
        user_id,
        mount_id,
        oauth2_access_token,
        oauth2_refresh_token,
        create_vault_repo,
        ..
    }: FakeRemoteAppConfig,
) {
    let _ = users_service.create_user(Some(user_id.clone()), Some(mount_id.clone()));

    {
        let mut state = state.write().unwrap();

        state.default_user_id = Some(user_id.clone());

        state
            .oauth2_access_tokens
            .insert(oauth2_access_token.clone(), user_id.to_owned());
        state
            .oauth2_refresh_tokens
            .insert(oauth2_refresh_token.clone(), user_id.to_owned());
    }

    if create_vault_repo {
        let context = Context {
            user_id: user_id.clone(),
            user_agent: None,
        };

        vault_repos_create_service
            .create_test_vault_repo(&context)
            .await
            .unwrap();
    }
}

async fn reset(app_state: AppState, config: FakeRemoteAppConfig) {
    app_state.state.write().unwrap().reset();

    init_state(&app_state, config).await;
}
