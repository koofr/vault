use axum::{body::Body, http::Request, response::Response};
use futures::future::BoxFuture;
use http::header;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct FixResponseJsonLayer;

impl<S> Layer<S> for FixResponseJsonLayer {
    type Service = FixResponseJsonMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FixResponseJsonMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct FixResponseJsonMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for FixResponseJsonMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;

            if response
                .headers()
                .get(header::CONTENT_TYPE)
                .filter(|value| {
                    *value == header::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref())
                })
                .is_some()
            {
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::try_from("application/json; charset=utf-8").unwrap(),
                );
            }

            Ok(response)
        })
    }
}
