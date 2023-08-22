use axum::{
    extract::State,
    http::{request, Request},
    middleware::Next,
    response::Response,
};

use super::app_state::AppState;

pub enum InterceptorResult {
    Ignore,
    Transform(Box<dyn FnOnce(Response) -> Response + Send + Sync + 'static>),
    Response(Response),
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
                InterceptorResult::Response(response) => response,
            }
        }
        None => next.run(request).await,
    }
}
