use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use http::HeaderMap;

use crate::common::state::BoxAsyncRead;

use super::HttpError;

pub type HttpResponseBytesStream =
    Pin<Box<dyn Stream<Item = Result<Vec<u8>, HttpError>> + Send + Sync + 'static>>;

pub enum HttpRequestBody {
    Bytes(Vec<u8>),
    Reader(BoxAsyncRead),
}

#[derive(Default)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<HttpRequestBody>,
    pub on_body_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
    pub is_retriable: bool,
}

impl HttpRequest {
    pub fn set_base_url(&mut self, base_url: &str) {
        self.url = format!("{}{}", base_url, self.url);
    }

    pub fn try_clone(&self) -> Option<Self> {
        let on_body_progress = match &self.on_body_progress {
            Some(_) => return None,
            None => None,
        };

        let body = match &self.body {
            Some(HttpRequestBody::Bytes(bytes)) => Some(HttpRequestBody::Bytes(bytes.clone())),
            Some(HttpRequestBody::Reader(_)) => return None,
            None => None,
        };

        Some(Self {
            method: self.method.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            body,
            on_body_progress,
            is_retriable: self.is_retriable,
        })
    }
}

#[async_trait]
pub trait HttpResponse {
    fn status_code(&self) -> u16;
    fn headers(&self) -> &HeaderMap;
    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, HttpError>;
    fn bytes_stream(self: Box<Self>) -> HttpResponseBytesStream;
}

pub type BoxHttpResponse = Box<dyn HttpResponse + Send + Sync>;

#[async_trait]
pub trait HttpClient {
    async fn request(&self, request: HttpRequest) -> Result<BoxHttpResponse, HttpError>;
}
