use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum_server::{tls_rustls::RustlsConfig, Handle};
use tokio::{sync::oneshot, task::JoinHandle};

use super::{app_state::AppState, errors::FakeRemoteServerStartError, router::build_router};

pub struct FakeRemoteServer {
    app_state: AppState,
    proposed_addr: SocketAddr,
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    tokio_runtime: Arc<tokio::runtime::Runtime>,

    handle: Arc<RwLock<Option<Handle>>>,
    addr: Arc<RwLock<Option<SocketAddr>>>,
    serve_join_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    start_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl FakeRemoteServer {
    pub fn new(
        app_state: AppState,
        proposed_addr: Option<SocketAddr>,
        cert_pem: Vec<u8>,
        key_pem: Vec<u8>,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    ) -> Self {
        let proposed_addr = proposed_addr.unwrap_or(SocketAddr::from(([127, 0, 0, 1], 0)));

        Self {
            app_state,
            proposed_addr,
            cert_pem,
            key_pem,
            tokio_runtime,

            handle: Arc::new(RwLock::new(None)),
            addr: Arc::new(RwLock::new(None)),
            serve_join_handle: Arc::new(RwLock::new(None)),
            start_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub async fn start(&self) -> Result<String, FakeRemoteServerStartError> {
        let start_guard = self.start_mutex.lock().await;

        if let Some(addr) = self.addr.read().unwrap().as_ref() {
            return Err(FakeRemoteServerStartError::AlreadyStarted(addr_to_url(
                *addr,
            )));
        }

        let proposed_addr = self.proposed_addr;

        let rustls_config = RustlsConfig::from_pem(self.cert_pem.clone(), self.key_pem.clone())
            .await
            .map_err(|err| FakeRemoteServerStartError::InvalidCertOrKey(Arc::new(err)))?;

        let router = build_router(self.app_state.clone());

        let handle = Handle::new();

        *self.handle.write().unwrap() = Some(handle.clone());

        let (serve_error_tx, serve_error_rx) = oneshot::channel();

        let serve_handle = handle.clone();

        let serve_join_handle = self.tokio_runtime.spawn(async move {
            if let Err(err) = axum_server::bind_rustls(proposed_addr, rustls_config)
                .handle(serve_handle)
                .serve(router.into_make_service())
                .await
            {
                let _ = serve_error_tx.send(err);
            }
        });

        match handle.listening().await {
            Some(addr) => {
                *self.addr.write().unwrap() = Some(addr);

                log::info!("FakeRemoteServer listening on {}", addr);

                *self.serve_join_handle.write().unwrap() = Some(serve_join_handle);

                drop(start_guard);

                Ok(addr_to_url(addr))
            }
            None => {
                let _ = serve_join_handle.await;

                let err = serve_error_rx.await.unwrap();

                log::warn!("FakeRemoteServer listen error: {:?}", err);

                drop(start_guard);

                Err(FakeRemoteServerStartError::ListenError(Arc::new(err)))
            }
        }
    }

    pub async fn ensure_started(&self) -> Result<String, FakeRemoteServerStartError> {
        match self.start().await {
            Ok(proxy_url) => Ok(proxy_url),
            Err(FakeRemoteServerStartError::AlreadyStarted(proxy_url)) => Ok(proxy_url),
            Err(err) => Err(err),
        }
    }

    pub async fn stop(&self) {
        let start_guard = self.start_mutex.lock().await;

        if let Some(handle) = self.handle.write().unwrap().take() {
            // TODO graceful shutdown
            handle.shutdown();
        }

        self.addr.write().unwrap().take();

        let join_handle = self.serve_join_handle.write().unwrap().take();

        if let Some(join_handle) = join_handle {
            let _ = join_handle.await;
        }

        drop(start_guard);
    }
}

pub fn addr_to_url(addr: SocketAddr) -> String {
    format!("https://{}", addr)
}
