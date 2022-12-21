use std::pin::Pin;

use async_trait::async_trait;
use futures::{
    future::{BoxFuture, Shared},
    AsyncRead, Stream,
};
use http::HeaderMap;

use super::HttpError;

pub type HttpRequestBodyReader = Pin<Box<dyn AsyncRead + Send + Sync + 'static>>;

pub type HttpResponseBytesStream =
    Pin<Box<dyn Stream<Item = Result<Vec<u8>, HttpError>> + Send + Sync + 'static>>;

pub type HttpRequestAbort = Option<Shared<BoxFuture<'static, Result<(), ()>>>>;

pub enum HttpRequestBody {
    Bytes(Vec<u8>),
    Reader(Pin<Box<dyn AsyncRead + Send + Sync + 'static>>),
}

#[derive(Default)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Option<HttpRequestBody>,
    pub on_body_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
    pub abort: HttpRequestAbort,
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
