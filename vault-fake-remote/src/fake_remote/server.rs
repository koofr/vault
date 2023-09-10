use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::Router;
use axum_server::{tls_rustls::RustlsConfig, Handle};
use futures::FutureExt;
use tokio::{sync::oneshot, task::JoinHandle};

use super::errors::FakeRemoteServerStartError;

#[derive(Debug)]
pub enum FakeRemoteServerListener {
    Http {
        proposed_addr: SocketAddr,
    },
    Https {
        proposed_addr: SocketAddr,
        cert_pem: Vec<u8>,
        key_pem: Vec<u8>,
    },
}

impl FakeRemoteServerListener {
    pub fn addr_to_url(&self, addr: SocketAddr) -> String {
        match self {
            Self::Http { .. } => format!("http://{}", addr),
            Self::Https { .. } => format!("https://{}", addr),
        }
    }
}

pub struct FakeRemoteServer {
    listener: FakeRemoteServerListener,
    tokio_runtime: Arc<tokio::runtime::Runtime>,

    handle: Arc<RwLock<Option<Handle>>>,
    addr: Arc<RwLock<Option<SocketAddr>>>,
    serve_join_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    start_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl FakeRemoteServer {
    pub fn new(
        listener: FakeRemoteServerListener,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    ) -> Self {
        Self {
            listener,
            tokio_runtime,

            handle: Arc::new(RwLock::new(None)),
            addr: Arc::new(RwLock::new(None)),
            serve_join_handle: Arc::new(RwLock::new(None)),
            start_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    pub async fn start(&self, router: Router) -> Result<String, FakeRemoteServerStartError> {
        let start_guard = self.start_mutex.lock().await;

        if let Some(addr) = self.addr.read().unwrap().as_ref() {
            return Err(FakeRemoteServerStartError::AlreadyStarted(
                self.listener.addr_to_url(*addr),
            ));
        }

        let handle = Handle::new();

        *self.handle.write().unwrap() = Some(handle.clone());

        let (serve_error_tx, serve_error_rx) = oneshot::channel();

        let serve_handle = handle.clone();

        let serve_future = match &self.listener {
            FakeRemoteServerListener::Http { proposed_addr } => {
                axum_server::bind(proposed_addr.to_owned())
                    .handle(serve_handle)
                    .serve(router.into_make_service())
                    .boxed()
            }
            FakeRemoteServerListener::Https {
                proposed_addr,
                cert_pem,
                key_pem,
            } => {
                let rustls_config = RustlsConfig::from_pem(cert_pem.clone(), key_pem.clone())
                    .await
                    .map_err(|err| FakeRemoteServerStartError::InvalidCertOrKey(Arc::new(err)))?;

                axum_server::bind_rustls(proposed_addr.to_owned(), rustls_config)
                    .handle(serve_handle)
                    .serve(router.into_make_service())
                    .boxed()
            }
        };

        let serve_join_handle = self.tokio_runtime.spawn(async move {
            if let Err(err) = serve_future.await {
                let _ = serve_error_tx.send(err);
            }
        });

        match handle.listening().await {
            Some(addr) => {
                *self.addr.write().unwrap() = Some(addr);

                log::info!("FakeRemoteServer listening on {}", addr);

                *self.serve_join_handle.write().unwrap() = Some(serve_join_handle);

                drop(start_guard);

                Ok(self.listener.addr_to_url(addr))
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

    pub async fn ensure_started(
        &self,
        router: Router,
    ) -> Result<String, FakeRemoteServerStartError> {
        match self.start(router).await {
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
