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
}

#[async_trait]
pub trait HttpResponse {
    fn status_code(&self) -> u16;
    fn headers(&self) -> &HeaderMap;
    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, HttpError>;
    fn bytes_stream(self: Box<Self>) -> HttpResponseBytesStream;
}

#[async_trait]
pub trait HttpClient {
    async fn request(
        &self,
        request: HttpRequest,
    ) -> Result<Box<dyn HttpResponse + Send + Sync>, HttpError>;
}
