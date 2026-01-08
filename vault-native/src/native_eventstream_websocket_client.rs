use std::{pin::Pin, str::FromStr, sync::Arc};

use futures::{Sink, SinkExt, StreamExt, lock::Mutex};
use http::Uri;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{Error, Message},
};

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
        let url = Uri::from_str(&url).unwrap();
        let write_mutex = self.write.clone();
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

            let (write, read) = ws_stream.split();

            *write_mutex.lock().await = Some(Box::pin(write));

            on_open();

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
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier {}))
        .with_no_client_auth();

    let connector = tokio_tungstenite::Connector::Rustls(Arc::new(config));

    Some(connector)
}

#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
            rustls::SignatureScheme::ED448,
        ]
    }
}
