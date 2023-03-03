use async_trait::async_trait;
use futures::stream;
use http::HeaderMap;

use super::{HttpClient, HttpError, HttpRequest, HttpResponse, HttpResponseBytesStream};

pub struct MockHttpClient {
    on_request: Box<dyn Fn(HttpRequest) -> Result<MockHttpResponse, HttpError> + Send + Sync>,
}

impl MockHttpClient {
    pub fn new(
        on_request: Box<dyn Fn(HttpRequest) -> Result<MockHttpResponse, HttpError> + Send + Sync>,
    ) -> Self {
        Self { on_request }
    }
}

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn request(
        &self,
        http_request: HttpRequest,
    ) -> Result<Box<dyn HttpResponse + Send + Sync>, HttpError> {
        (self.on_request)(http_request)
            .map(|res| Box::new(res) as Box<dyn HttpResponse + Send + Sync>)
    }
}

pub struct MockHttpResponse {
    pub status_code: u16,
    pub headers: HeaderMap,
    pub bytes: Vec<u8>,
    pub bytes_stream: Option<HttpResponseBytesStream>,
}

impl MockHttpResponse {
    pub fn new(status_code: u16, headers: HeaderMap, bytes: Vec<u8>) -> Self {
        Self {
            status_code,
            headers,
            bytes,
            bytes_stream: None,
        }
    }
}

#[async_trait]
impl HttpResponse for MockHttpResponse {
    fn status_code(&self) -> u16 {
        self.status_code
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, HttpError> {
        Ok(self.bytes.clone())
    }

    fn bytes_stream(self: Box<Self>) -> HttpResponseBytesStream {
        self.bytes_stream.unwrap_or_else(|| {
            let bytes = self.bytes.clone();

            Box::pin(stream::once(async { Ok(bytes) }))
        })
    }
}
