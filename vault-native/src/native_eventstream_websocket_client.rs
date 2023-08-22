use std::{pin::Pin, sync::Arc};

use futures::{lock::Mutex, Sink, SinkExt, StreamExt};
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{Error, Message},
};
use url::Url;

use vault_core::eventstream::WebSocketClient;

pub struct NativeEventstreamWebSocketClient {
    tokio_runtime: Arc<tokio::runtime::Runtime>,
    tokio_tungstenite_connector: Option<tokio_tungstenite::Connector>,

    write: Arc<Mutex<Option<Pin<Box<dyn Sink<Message, Error = Error> + Send + Sync + 'static>>>>>,
}

impl NativeEventstreamWebSocketClient {
    pub fn new(
        tokio_runtime: Arc<tokio::runtime::Runtime>,
        tokio_tungstenite_connector: Option<tokio_tungstenite::Connector>,
    ) -> NativeEventstreamWebSocketClient {
        NativeEventstreamWebSocketClient {
            tokio_runtime,
            tokio_tungstenite_connector,

            write: Arc::new(Mutex::new(None)),
        }
    }
}

impl WebSocketClient for NativeEventstreamWebSocketClient {
    fn open(
        &self,
        url: String,
        on_open: Box<dyn Fn() + Send + Sync + 'static>,
        on_message: Box<dyn Fn(String) + Send + Sync + 'static>,
        on_close: Box<dyn Fn() + Send + Sync + 'static>,
    ) {
        let url = Url::parse(&url).unwrap();
        let spawn_self = self.write.clone();
        let tokio_tungstenite_connector = self.tokio_tungstenite_connector.clone();

        self.tokio_runtime.spawn(async move {
            let (ws_stream, _) =
                match connect_async_tls_with_config(url, None, true, tokio_tungstenite_connector)
                    .await
                {
                    Ok(stream) => stream,
                    Err(err) => {
                        log::debug!("NativeEventstreamWebSocketClient connect error: {:?}", err);

                        on_close();

                        return;
                    }
                };

            on_open();

            let (write, read) = ws_stream.split();

            *spawn_self.lock().await = Some(Box::pin(write));

            let on_message = Arc::new(Mutex::new(on_message));

            read.for_each(|message| async {
                match message {
                    Ok(message) => {
                        if let Ok(text) = message.to_text() {
                            (on_message.lock().await)(text.to_owned());
                        }
                    }
                    Err(err) => {
                        log::debug!("NativeEventstreamWebSocketClient message error: {:?}", err);
                    }
                }
            })
            .await;

            on_close();
        });
    }

    fn send(&self, data: String) {
        let write = self.write.clone();

        self.tokio_runtime.spawn(Box::pin(async move {
            let mut write = write.lock().await;

            if let Some(write) = write.as_mut() {
                let _ = write.send(Message::from(data)).await;
            }
        }));
    }

    fn close(&self) {
        let write = self.write.clone();

        self.tokio_runtime.spawn(Box::pin(async move {
            let mut write = write.lock().await;

            if let Some(write) = write.as_mut() {
                let _ = write.close().await;
            }
        }));
    }
}

pub fn get_tokio_tungstenite_connector(
    accept_invalid_certs: bool,
) -> Option<tokio_tungstenite::Connector> {
    if !accept_invalid_certs {
        // use default connector
        return None;
    }

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(NoVerifier {}))
        .with_no_client_auth();

    let connector = tokio_tungstenite::Connector::Rustls(Arc::new(config));

    Some(connector)
}

struct NoVerifier;

impl rustls::client::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::HandshakeSignatureValid::assertion())
    }
}
