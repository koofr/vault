use std::{
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};

use vault_fake_remote::fake_remote::{
    self,
    app_state::AppState,
    errors::FakeRemoteServerStartError,
    eventstream,
    files::service::FilesService,
    interceptor::{Interceptor, InterceptorResult},
    state::FakeRemoteState,
    FakeRemoteServer,
};

struct InterceptorContainer {
    interceptor: Arc<Mutex<Option<Interceptor>>>,
}

pub struct FakeRemote {
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,

    pub server: Arc<FakeRemoteServer>,
    pub data_path: PathBuf,
    pub state: Arc<RwLock<FakeRemoteState>>,
    pub files_service: Arc<FilesService>,

    interceptor_container: Arc<InterceptorContainer>,
}

impl FakeRemote {
    pub fn new(tokio_runtime: Arc<tokio::runtime::Runtime>) -> Self {
        let data_path = std::env::temp_dir().join(format!(
            "vault-core-tests-fake-remote-data-{}",
            &uuid::Uuid::new_v4().to_string()[..8]
        ));

        std::fs::create_dir_all(&data_path).unwrap();

        let state = Arc::new(RwLock::new(FakeRemoteState::default()));
        let eventstream_listeners = Arc::new(eventstream::Listeners::new());
        let files_service = Arc::new(FilesService::new(
            state.clone(),
            eventstream_listeners.clone(),
            data_path.clone(),
        ));

        let interceptor_container = Arc::new(InterceptorContainer {
            interceptor: Arc::new(Mutex::new(None)),
        });

        let interceptor_interceptor_container = interceptor_container.clone();
        let interceptor: Interceptor =
            Box::new(move |parts| {
                match interceptor_interceptor_container
                    .interceptor
                    .lock()
                    .unwrap()
                    .as_ref()
                {
                    Some(interceptor) => interceptor(parts),
                    None => InterceptorResult::Ignore,
                }
            });

        let app_state = AppState {
            state: state.clone(),
            files_service: files_service.clone(),
            eventstream_listeners: eventstream_listeners.clone(),
            interceptor: Arc::new(Some(interceptor)),
        };

        let server = Arc::new(FakeRemoteServer::new(
            app_state,
            None,
            fake_remote::CERT_PEM.to_owned(),
            fake_remote::KEY_PEM.to_owned(),
            tokio_runtime.clone(),
        ));

        Self {
            tokio_runtime,

            server,
            data_path,
            state,
            files_service,

            interceptor_container,
        }
    }

    pub async fn start(&self) -> Result<String, FakeRemoteServerStartError> {
        self.server.start().await
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

        std::fs::remove_dir_all(&self.data_path).unwrap();
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
