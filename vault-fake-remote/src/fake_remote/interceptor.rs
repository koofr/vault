use std::time::Duration;

use axum::{
    extract::State,
    http::{request, Request},
    middleware::Next,
    response::Response,
};
use futures::future::BoxFuture;
use vault_core::utils::{abort_http_body::AbortHttpBody, delayed_http_body::DelayedHttpBody};

use super::app_state::AppState;

pub enum InterceptorResult {
    Ignore,
    Transform(Box<dyn FnOnce(Response) -> Response + Send + Sync + 'static>),
    AsyncTransform(
        Box<dyn FnOnce(Response) -> BoxFuture<'static, Response> + Send + Sync + 'static>,
    ),
    Response(Response),
    AsyncResponse(BoxFuture<'static, Response>),
}

impl InterceptorResult {
    pub fn delayed_response_body(duration: Duration) -> Self {
        Self::Transform(Box::new(move |response| {
            let (parts, body) = response.into_parts();
            let body = DelayedHttpBody::unsync_box_body(body, tokio::time::sleep(duration));
            Response::from_parts(parts, body)
        }))
    }

    pub fn delayed_abort_response_body(duration: Duration) -> Self {
        Self::Transform(Box::new(move |response| {
            let (parts, _) = response.into_parts();
            let body = AbortHttpBody::new(|| {
                axum::Error::new(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
            });
            let body = DelayedHttpBody::unsync_box_body(body, tokio::time::sleep(duration));
            Response::from_parts(parts, body)
        }))
    }
}

pub type Interceptor = Box<dyn Fn(&request::Parts) -> InterceptorResult + Send + Sync + 'static>;

pub async fn interceptor_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    match state.interceptor.as_ref() {
        Some(interceptor) => {
            let (parts, body) = request.into_parts();

            let res = interceptor(&parts);

            let request = Request::from_parts(parts, body);

            match res {
                InterceptorResult::Ignore => next.run(request).await,
                InterceptorResult::Transform(transform) => transform(next.run(request).await),
                InterceptorResult::AsyncTransform(transform) => {
                    transform(next.run(request).await).await
                }
                InterceptorResult::Response(response) => response,
                InterceptorResult::AsyncResponse(response) => response.await,
            }
        }
        None => next.run(request).await,
    }
}
