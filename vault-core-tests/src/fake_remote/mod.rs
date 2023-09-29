use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use vault_fake_remote::fake_remote::{
    self,
    app_state::AppState,
    errors::FakeRemoteServerStartError,
    interceptor::{Interceptor, InterceptorResult},
    router::build_router,
    server::{FakeRemoteServer, FakeRemoteServerListener},
};

struct InterceptorContainer {
    interceptor: Arc<Mutex<Option<Interceptor>>>,
}

pub struct FakeRemote {
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,

    pub app_state: AppState,
    pub server: Arc<FakeRemoteServer>,

    interceptor_container: Arc<InterceptorContainer>,
}

impl FakeRemote {
    pub fn new(tokio_runtime: Arc<tokio::runtime::Runtime>) -> Self {
        let data_path = std::env::temp_dir().join(format!(
            "vault-core-tests-fake-remote-data-{}",
            &uuid::Uuid::new_v4().to_string()[..8]
        ));

        std::fs::create_dir_all(&data_path).unwrap();

        let mut app_state = AppState::new(data_path.clone());

        let interceptor_container = Arc::new(InterceptorContainer {
            interceptor: Arc::new(Mutex::new(None)),
        });

        let interceptor_interceptor_container = interceptor_container.clone();

        app_state.interceptor =
            Arc::new(Some(Box::new(
                move |parts| match interceptor_interceptor_container
                    .interceptor
                    .lock()
                    .unwrap()
                    .as_ref()
                {
                    Some(interceptor) => interceptor(parts),
                    None => InterceptorResult::Ignore,
                },
            )));

        let server = Arc::new(FakeRemoteServer::new(
            FakeRemoteServerListener::Https {
                proposed_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
                cert_pem: fake_remote::CERT_PEM.to_owned(),
                key_pem: fake_remote::KEY_PEM.to_owned(),
            },
            tokio_runtime.clone(),
        ));

        Self {
            tokio_runtime,

            app_state,
            server,

            interceptor_container,
        }
    }

    pub async fn start(&self) -> Result<String, FakeRemoteServerStartError> {
        self.server
            .start(build_router(self.app_state.clone()))
            .await
    }

    pub async fn stop(&self) {
        self.server.stop().await
    }

    pub fn intercept(&self, interceptor: Interceptor) {
        *self.interceptor_container.interceptor.lock().unwrap() = Some(interceptor)
    }
}

impl Drop for FakeRemote {
    fn drop(&mut self) {
        let server = self.server.clone();

        self.tokio_runtime.spawn(async move { server.stop().await });

        std::fs::remove_dir_all(self.app_state.data_path.as_ref()).unwrap();
    }
}

pub fn log_interceptor() -> Interceptor {
    Box::new(|parts| {
        println!("REQUEST: {:?}", parts);

        InterceptorResult::Transform(Box::new(|response| {
            println!("RESPONSE: {:?}", response);

            response
        }))
    })
}
