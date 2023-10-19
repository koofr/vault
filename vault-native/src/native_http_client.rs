use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use futures::stream::{AbortHandle, StreamExt};
use http::{header, HeaderMap, HeaderValue};
use reqwest;

use vault_core::{
    http::{
        BoxHttpResponse, HttpClient, HttpError, HttpRequest, HttpRequestBody, HttpResponse,
        HttpResponseBytesStream,
    },
    utils::{
        abort_reader::AbortReader, drop_abort::DropAbort, on_end_reader::OnEndReader,
        progress_reader::ProgressReader, reader_stream::ReaderStream,
    },
};

pub fn get_reqwest_client(accept_invalid_certs: bool) -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .unwrap()
}

pub struct NativeHttpClient {
    client: Arc<reqwest::Client>,
    user_agent: HeaderValue,
}

impl NativeHttpClient {
    pub fn new(client: Arc<reqwest::Client>, user_agent: String) -> Self {
        let user_agent = HeaderValue::from_str(&user_agent).unwrap();

        Self { client, user_agent }
    }
}

#[async_trait]
impl HttpClient for NativeHttpClient {
    async fn request(&self, http_request: HttpRequest) -> Result<BoxHttpResponse, HttpError> {
        let mut http_request = http_request;

        let method = reqwest::Method::from_bytes(http_request.method.as_bytes()).unwrap();

        let mut headers = http_request.headers;

        headers.insert(header::USER_AGENT, self.user_agent.clone());

        let mut req = self
            .client
            .request(method, http_request.url)
            .headers(headers);

        let (abort_handle, _) = AbortHandle::new_pair();
        let drop_abort = DropAbort(abort_handle.clone());

        let reader_error: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
        let on_end_reader_reader_error = reader_error.clone();

        req = match http_request.body {
            Some(HttpRequestBody::Bytes(bytes)) => req.body(bytes),
            Some(HttpRequestBody::Reader(reader)) => {
                // we need a custom abort reader. for some reason reqwest does
                // not cancel the request on abort
                let abort_reader = AbortReader::new(reader, abort_handle.clone());

                let on_body_progress = Arc::new(http_request.on_body_progress.take());

                let progress_reader = ProgressReader::new(
                    abort_reader,
                    Box::new(move |n| match on_body_progress.as_deref() {
                        Some(on_body_progress) => on_body_progress(n),
                        None => {}
                    }),
                );

                let on_end_reader = OnEndReader::new(
                    progress_reader,
                    Box::new(move |res| {
                        if let Err(err) = res {
                            *on_end_reader_reader_error.write().unwrap() = Some(err.to_string())
                        }
                    }),
                );

                req.body(reqwest::Body::wrap_stream(ReaderStream::new(
                    on_end_reader,
                    vault_core::cipher::constants::BLOCK_SIZE,
                )))
            }
            None => req,
        };

        let response = req.send().await.map_err(|err| {
            let mut err_str = err.to_string();

            if let Some(reader_err) = reader_error.read().unwrap().as_ref() {
                err_str = format!("{}: {}", err_str, reader_err)
            }

            HttpError::ResponseError(err_str)
        })?;

        // this needs to be after response for the abort to work
        let _ = drop_abort;

        Ok(Box::new(ReqwestHttpResponse { response }))
    }
}

pub struct ReqwestHttpResponse {
    response: reqwest::Response,
}

#[async_trait]
impl HttpResponse for ReqwestHttpResponse {
    fn status_code(&self) -> u16 {
        self.response.status().as_u16()
    }

    fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, HttpError> {
        self.response
            .bytes()
            .await
            .map(|bytes| bytes.into())
            .map_err(|e| HttpError::ResponseError(e.to_string()))
    }

    fn bytes_stream(self: Box<Self>) -> HttpResponseBytesStream {
        Box::pin(self.response.bytes_stream().map(|bytes| {
            bytes
                .map(|bytes| bytes.into())
                .map_err(|error| HttpError::ResponseError(error.to_string()))
        }))
    }
}
