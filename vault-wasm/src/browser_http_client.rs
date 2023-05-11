use std::{sync::Arc, task::Poll};

use async_trait::async_trait;
use futures::{stream::poll_fn, Future, Stream, StreamExt};
use http::{header::HeaderName, HeaderMap};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::{AbortController, AbortSignal, Request, RequestInit, Response};

use vault_core::{
    cipher::constants::BLOCK_SIZE,
    http::{
        HttpClient, HttpError, HttpRequest, HttpRequestBody, HttpResponse, HttpResponseBytesStream,
    },
    utils::progress_reader::ProgressReader,
};

use crate::helpers;

#[wasm_bindgen(typescript_custom_section)]
const BROWSER_HTTP_CLIENT_DELEGATE: &'static str = r#"
export interface BrowserHttpClientDelegate {
  fetch(request: Request): Promise<Response>;
  xhr(
    request: Request,
    blob: Blob,
    onRequestProgress: (n: number) => void
  ): Promise<Response>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BrowserHttpClientDelegate")]
    pub type BrowserHttpClientDelegate;

    #[wasm_bindgen(structural, method)]
    pub fn fetch(this: &BrowserHttpClientDelegate, request: &Request) -> js_sys::Promise;

    #[wasm_bindgen(structural, method)]
    pub fn xhr(
        this: &BrowserHttpClientDelegate,
        request: &Request,
        blob: &JsValue,
        on_request_progress: &Closure<dyn Fn(usize)>,
    ) -> js_sys::Promise;
}

pub struct BrowserHttpClient {
    browser_http_client_delegate: BrowserHttpClientDelegate,
}

unsafe impl Send for BrowserHttpClient {}
unsafe impl Sync for BrowserHttpClient {}

impl BrowserHttpClient {
    pub fn new(browser_http_client_delegate: BrowserHttpClientDelegate) -> BrowserHttpClient {
        BrowserHttpClient {
            browser_http_client_delegate,
        }
    }

    fn needs_xhr(&self, http_request: &HttpRequest) -> bool {
        match &http_request.body {
            Some(HttpRequestBody::Reader(_)) => !helpers::supports_request_streams(),
            _ => false,
        }
    }

    async fn get_request(&self, http_request: HttpRequest, abort_signal: AbortSignal) -> Request {
        let mut http_request = http_request;

        let mut opts = RequestInit::new();

        opts.method(&http_request.method.clone());

        opts.signal(Some(&abort_signal));

        match http_request.body {
            Some(HttpRequestBody::Bytes(bytes)) => {
                let blob = helpers::bytes_to_blob(&bytes, None);

                opts.body(Some(&blob));
            }
            Some(HttpRequestBody::Reader(reader)) => {
                let on_body_progress = Arc::new(http_request.on_body_progress.take());

                let progress_reader = ProgressReader::new(
                    reader,
                    Box::new(move |n| match on_body_progress.as_deref() {
                        Some(on_body_progress) => on_body_progress(n),
                        None => {}
                    }),
                );

                let stream =
                    ReadableStream::from_async_read(progress_reader, BLOCK_SIZE).into_raw();
                let stream_value = JsValue::from(stream);

                opts.body(Some(&stream_value));

                js_sys::Reflect::set(
                    opts.as_ref(),
                    &JsValue::from("duplex"),
                    &JsValue::from("half"),
                )
                .unwrap();
            }
            None => {}
        };

        let request = Request::new_with_str_and_init(&http_request.url, &opts).unwrap();

        let headers = request.headers();

        for (key, value) in http_request.headers.iter() {
            headers
                .set(&key.as_str(), &value.to_str().unwrap())
                .unwrap();
        }

        request
    }

    pub async fn get_response(
        &self,
        response_promise: js_sys::Promise,
    ) -> Result<Response, HttpError> {
        let resp_value = JsFuture::from(response_promise)
            .await
            .map_err(|err| HttpError::ResponseError(helpers::error_string(&err)))?;

        let response: Response = resp_value.dyn_into().unwrap();

        Ok(response.into())
    }

    pub async fn request_fetch(
        &self,
        http_request: HttpRequest,
        abort_signal: AbortSignal,
    ) -> Result<Response, HttpError> {
        let request = self.get_request(http_request, abort_signal).await;

        self.get_response(self.browser_http_client_delegate.fetch(&request))
            .await
    }

    pub async fn request_xhr(
        &self,
        http_request: HttpRequest,
        abort_signal: AbortSignal,
    ) -> Result<Response, HttpError> {
        let mut http_request = http_request;

        let on_body_progress = Arc::new(http_request.on_body_progress.take());

        let body = http_request.body.take();

        let blob = match body {
            Some(HttpRequestBody::Bytes(bytes)) => helpers::bytes_to_blob(&bytes, None),
            Some(HttpRequestBody::Reader(reader)) => helpers::reader_to_blob(reader, None)
                .await
                .map_err(|err| HttpError::ResponseError(err.to_string()))?,
            None => JsValue::UNDEFINED,
        };

        let request = self.get_request(http_request, abort_signal).await;

        let on_request_progress_closure =
            Closure::new(Box::new(move |n| match on_body_progress.as_deref() {
                Some(on_body_progress) => on_body_progress(n),
                None => {}
            }));

        self.get_response(self.browser_http_client_delegate.xhr(
            &request,
            &blob,
            &on_request_progress_closure,
        ))
        .await
    }

    pub async fn request_js(
        &self,
        http_request: HttpRequest,
    ) -> Result<Box<dyn HttpResponse>, HttpError> {
        let abort = AbortGuard::new();

        let resp = if self.needs_xhr(&http_request) {
            self.request_xhr(http_request, abort.signal()).await?
        } else {
            self.request_fetch(http_request, abort.signal()).await?
        };

        Ok(Box::new(FetchHttpResponse::new(resp, abort)))
    }
}

#[async_trait]
impl HttpClient for BrowserHttpClient {
    async fn request(
        &self,
        http_request: HttpRequest,
    ) -> Result<Box<dyn HttpResponse + Send + Sync>, HttpError> {
        let future = Box::into_pin(unsafe {
            Box::from_raw(Box::into_raw(Box::new(self.request_js(http_request))
                as Box<dyn Future<Output = Result<Box<dyn HttpResponse>, HttpError>>>)
                as *mut (dyn Future<Output = Result<Box<dyn HttpResponse + Send + Sync>, HttpError>>
                     + Send
                     + Sync))
        });

        future.await
    }
}

pub fn get_response_headers(resp: &Response) -> HeaderMap {
    let resp_headers_array = js_sys::Array::from(&resp.headers().into());

    let mut headers = HeaderMap::with_capacity(resp_headers_array.length().try_into().unwrap());

    for i in 0..resp_headers_array.length() {
        let tuple: js_sys::Array = resp_headers_array.get(i).dyn_into().unwrap();
        let key = tuple.get(0).as_string().unwrap();
        let value = tuple.get(1).as_string().unwrap();
        headers.insert(
            HeaderName::from_bytes(key.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }

    headers
}

pub struct FetchHttpResponse {
    response: Response,
    headers: HeaderMap,
    _abort: AbortGuard,
}

unsafe impl Send for FetchHttpResponse {}
unsafe impl Sync for FetchHttpResponse {}

impl FetchHttpResponse {
    fn new(response: Response, abort: AbortGuard) -> Self {
        let headers = get_response_headers(&response);

        Self {
            response,
            headers,
            _abort: abort,
        }
    }

    async fn bytes_js(self: Box<Self>) -> Result<Vec<u8>, HttpError> {
        let buf = JsFuture::from(JsFuture::from(self.response.array_buffer().unwrap()))
            .await
            .unwrap();
        assert!(buf.is_instance_of::<js_sys::ArrayBuffer>());
        let array: js_sys::Uint8Array = js_sys::Uint8Array::new(&buf);
        let body = array.to_vec();

        Ok(body)
    }

    fn bytes_stream_js(self: Box<Self>) -> Box<dyn Stream<Item = Result<Vec<u8>, HttpError>>> {
        let abort = self._abort;

        let raw_body = self.response.body().unwrap();
        let body = ReadableStream::from_raw(raw_body.dyn_into().unwrap());

        let stream = body.into_stream();

        Box::new(
            stream
                .map(|chunk| {
                    chunk
                        .map(|value| value.dyn_into::<js_sys::Uint8Array>().unwrap().to_vec())
                        .map_err(|err| HttpError::ResponseError(helpers::error_string(&err)))
                })
                .chain(poll_fn(move |_| {
                    // Keep the abort guard alive as long as this stream is,
                    // otherwise the request/response might get aborted before
                    // we read the whole response body.
                    let _ = &abort;

                    Poll::Ready(None)
                })),
        )
    }
}

#[async_trait]
impl HttpResponse for FetchHttpResponse {
    fn status_code(&self) -> u16 {
        self.response.status()
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    async fn bytes(self: Box<Self>) -> Result<Vec<u8>, HttpError> {
        let future = Box::into_pin(unsafe {
            Box::from_raw(Box::into_raw(
                Box::new(self.bytes_js()) as Box<dyn Future<Output = Result<Vec<u8>, HttpError>>>
            )
                as *mut (dyn Future<Output = Result<Vec<u8>, HttpError>> + Send))
        });

        future.await
    }

    fn bytes_stream(self: Box<Self>) -> HttpResponseBytesStream {
        Box::into_pin(unsafe {
            Box::from_raw(Box::into_raw(self.bytes_stream_js())
                as *mut (dyn Stream<Item = Result<Vec<u8>, HttpError>> + Send + Sync))
        })
    }
}

// from https://github.com/seanmonstar/reqwest/blob/eeca649a3d70c353043b2e42684c6d74f4ba5cae/src/wasm/mod.rs
struct AbortGuard {
    ctrl: AbortController,
}

impl AbortGuard {
    fn new() -> Self {
        Self {
            ctrl: AbortController::new().unwrap(),
        }
    }

    fn signal(&self) -> AbortSignal {
        self.ctrl.signal()
    }
}

impl Drop for AbortGuard {
    fn drop(&mut self) {
        self.ctrl.abort();
    }
}
