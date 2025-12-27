use axum::{extract::Request, middleware::Next, response::Response};
use http::header;

pub async fn fix_response_json(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    if response
        .headers()
        .get(header::CONTENT_TYPE)
        .filter(|value| *value == header::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()))
        .is_some()
    {
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::try_from("application/json; charset=utf-8").unwrap(),
        );
    }

    response
}
