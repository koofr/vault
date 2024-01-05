use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{header, request, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};

use crate::{encryption::Encryption, request_id::RequestId, sessions::Sessions};

#[derive(Clone)]
pub struct EncryptionMiddlewareState {
    pub encryption: Arc<Encryption>,
    pub sessions: Arc<Sessions>,
}

pub async fn encryption_middleware(
    State(state): State<EncryptionMiddlewareState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    if req.uri().path() == "/oauth2callback" {
        return Ok(next.run(req).await);
    }

    match *req.method() {
        Method::GET => {
            let req = decrypt_request_get(&state.encryption, &state.sessions, req).await?;

            Ok(next.run(req).await)
        }
        Method::POST => {
            let req = decrypt_request_post(&state.encryption, &state.sessions, req).await?;

            let res = next.run(req).await;

            encrypt_response(&state.encryption, res).await
        }
        Method::OPTIONS => Ok(next.run(req).await),
        _ => Err(StatusCode::METHOD_NOT_ALLOWED.into_response()),
    }
}

#[derive(Deserialize, Serialize)]
pub struct EncryptedRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Deserialize, Serialize)]
pub struct EncryptedQuery {
    pub req: String,
}

async fn decrypt_request_get(
    encryption: &Encryption,
    sessions: &Sessions,
    req: Request<Body>,
) -> Result<Request<Body>, Response> {
    let (parts, _) = req.into_parts();

    let query = parts
        .uri
        .query()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing query").into_response())?;
    let query: EncryptedQuery = serde_urlencoded::from_str(query)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    decrypt_request(encryption, sessions, query.req.as_bytes(), parts.headers).await
}

async fn decrypt_request_post(
    encryption: &Encryption,
    sessions: &Sessions,
    req: Request<Body>,
) -> Result<Request<Body>, Response> {
    let (parts, body) = req.into_parts();

    let body_bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?
        .to_bytes();

    decrypt_request(encryption, sessions, &body_bytes[..], parts.headers).await
}

async fn decrypt_request(
    encryption: &Encryption,
    sessions: &Sessions,
    encrypted_body: &[u8],
    raw_headers: header::HeaderMap<header::HeaderValue>,
) -> Result<Request<Body>, Response> {
    let decrypted_body = encryption
        .decrypt(encrypted_body)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    let encrypted_request: EncryptedRequest = serde_json::from_slice(&decrypted_body)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    if let Some(id) = encrypted_request.id.as_ref() {
        sessions
            .verify_request_id(id)
            .map_err(|err: crate::sessions::RequestSessionError| {
                (StatusCode::FORBIDDEN, err.to_string()).into_response()
            })?;
    }

    let method = Method::from_str(&encrypted_request.method)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    let uri = axum::http::Uri::builder()
        .path_and_query(encrypted_request.uri)
        .build()
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    let mut encrypted_headers = header::HeaderMap::new();

    for (key, value) in encrypted_request.headers {
        let key = header::HeaderName::from_str(&key)
            .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;
        let value = header::HeaderValue::from_str(&value)
            .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;
        encrypted_headers.insert(key, value);
    }

    let body_bytes = data_encoding::BASE64
        .decode(encrypted_request.body.as_bytes())
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    let mut builder = request::Builder::new().method(method).uri(uri);

    if let Some(id) = encrypted_request.id {
        builder = builder.extension(id);
    }

    if let Some(headers) = builder.headers_mut() {
        headers.extend(raw_headers);
        headers.extend(encrypted_headers);
        headers.remove(header::CONTENT_LENGTH);
    }

    let req = builder
        .body(Body::from(body_bytes))
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    Ok(req)
}

#[derive(Deserialize, Serialize)]
pub struct EncryptedResponse {
    pub headers: HashMap<String, String>,
    pub body: String,
}

async fn encrypt_response(encryption: &Encryption, res: Response) -> Result<Response, Response> {
    let (mut parts, body) = res.into_parts();

    let mut headers = HashMap::new();

    for key in [header::CONTENT_TYPE] {
        if let Some(value) = parts.headers.remove(&key) {
            headers.insert(key.as_str().to_owned(), value.to_str().unwrap().to_owned());
        }
    }

    parts.headers.remove(header::CONTENT_LENGTH);

    let body_bytes = body
        .collect()
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?
        .to_bytes();

    let body = data_encoding::BASE64.encode(&body_bytes);

    let encrypted_response = EncryptedResponse { headers, body };

    let body = serde_json::to_vec(&encrypted_response)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    let body = encryption
        .encrypt(&body)
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()).into_response())?;

    Ok((parts.status, parts.headers, body).into_response())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, convert::Infallible, sync::Arc};

    use axum::{
        body::Body,
        extract,
        http::{header, Method, Request, StatusCode},
        response::{IntoResponse, Response},
        routing::{get, post},
        Router,
    };
    use futures::StreamExt;
    use http_body_util::BodyExt;
    use similar_asserts::assert_eq;
    use tower::{Layer, Service, ServiceExt};

    use crate::{
        encryption::Encryption,
        request_id::RequestId,
        sessions::{SessionMessage, Sessions},
    };

    use super::{
        encryption_middleware, EncryptedRequest, EncryptedResponse, EncryptionMiddlewareState,
    };

    async fn get_handler(req: extract::Request) -> Response {
        format!("{} {}", req.method(), req.uri()).into_response()
    }

    async fn post_handler(req: extract::Request) -> Response {
        let (parts, body) = req.into_parts();

        let body_bytes = body.collect().await.unwrap().to_bytes();

        (
            StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                parts.headers.get(header::CONTENT_TYPE).unwrap().to_owned(),
            )],
            body_bytes,
        )
            .into_response()
    }

    fn build_app() -> (
        impl tower::Service<Request<Body>, Response = Response, Error = Infallible>,
        EncryptionMiddlewareState,
    ) {
        let encryption = Arc::new(
            Encryption::new_with_key_str("XrwBl00MUbeAZ4QBW2F+YDFBv80f2kes49VDx7wUs7Y=").unwrap(),
        );
        let sessions = Arc::new(Sessions::new());
        let state = EncryptionMiddlewareState {
            encryption,
            sessions,
        };
        let app = Router::new()
            .route("/get", get(get_handler))
            .route("/post", post(post_handler));
        let app =
            axum::middleware::from_fn_with_state(state.clone(), encryption_middleware).layer(app);

        (app, state)
    }

    #[tokio::test]
    async fn test_get() {
        let (app, state) = build_app();

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!(
                "/?req={}",
                urlencoding::encode(
                    &state
                        .encryption
                        .encrypt(
                            &serde_json::to_vec(&EncryptedRequest {
                                id: None,
                                method: "GET".into(),
                                uri: "/get?key=value".into(),
                                headers: Default::default(),
                                body: "".into(),
                            })
                            .unwrap(),
                        )
                        .unwrap()
                )
            ))
            .body(Body::empty())
            .unwrap();

        let (res, body) = app.oneshot(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(
            res.status,
            StatusCode::OK,
            "Expected status: {:?}: {}",
            res.status,
            String::from_utf8(body.to_vec()).unwrap()
        );
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "GET /get?key=value"
        );
    }

    #[tokio::test]
    async fn test_get_session_ok() {
        let (app, state) = build_app();

        let mut session_stream = state.sessions.create_session();

        let session_id = match session_stream.next().await.unwrap() {
            SessionMessage::Start { session_id } => session_id,
            _ => panic!("expected start"),
        };

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!(
                "/?req={}",
                urlencoding::encode(
                    &state
                        .encryption
                        .encrypt(
                            &serde_json::to_vec(&EncryptedRequest {
                                id: Some(RequestId {
                                    session_id,
                                    sequence_id: Some(0)
                                }),
                                method: "GET".into(),
                                uri: "/get?key=value".into(),
                                headers: Default::default(),
                                body: "".into(),
                            })
                            .unwrap(),
                        )
                        .unwrap()
                )
            ))
            .body(Body::empty())
            .unwrap();

        let (res, body) = app.oneshot(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(
            res.status,
            StatusCode::OK,
            "Expected status: {:?}: {}",
            res.status,
            String::from_utf8(body.to_vec()).unwrap()
        );
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "GET /get?key=value"
        );

        drop(session_stream);
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let (app, state) = build_app();

        let req = Request::builder()
            .method(Method::GET)
            .uri(format!(
                "/?req={}",
                urlencoding::encode(
                    &state
                        .encryption
                        .encrypt(
                            &serde_json::to_vec(&EncryptedRequest {
                                id: Some(RequestId {
                                    session_id: "00000000-0000-0000-0000-000000000000".into(),
                                    sequence_id: None,
                                }),
                                method: "GET".into(),
                                uri: "/get?key=value".into(),
                                headers: Default::default(),
                                body: "".into(),
                            })
                            .unwrap(),
                        )
                        .unwrap()
                )
            ))
            .body(Body::empty())
            .unwrap();

        let (res, body) = app.oneshot(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(res.status, StatusCode::FORBIDDEN);
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "session not found"
        );
    }

    #[tokio::test]
    async fn test_post() {
        let (app, state) = build_app();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::from(
                state
                    .encryption
                    .encrypt(
                        &serde_json::to_vec(&EncryptedRequest {
                            id: None,
                            method: "POST".into(),
                            uri: "/post?k=v".into(),
                            headers: HashMap::from([(
                                "Content-Type".into(),
                                "application/json".into(),
                            )]),
                            body: data_encoding::BASE64.encode(r#"{"key": "value"}"#.as_bytes()),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();

        let (res, body) = app.oneshot(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(
            res.status,
            StatusCode::OK,
            "Expected status: {:?}: {}",
            res.status,
            String::from_utf8(body.to_vec()).unwrap()
        );
        let body = state.encryption.decrypt(&body[..]).unwrap();
        let encrypted_response: EncryptedResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            encrypted_response.headers,
            HashMap::from([("content-type".into(), "application/json".into(),)])
        );
        let body = data_encoding::BASE64
            .decode(encrypted_response.body.as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(body).unwrap(), r#"{"key": "value"}"#);
    }

    #[tokio::test]
    async fn test_post_session_ok() {
        let (app, state) = build_app();

        let mut session_stream = state.sessions.create_session();

        let session_id = match session_stream.next().await.unwrap() {
            SessionMessage::Start { session_id } => session_id,
            _ => panic!("expected start"),
        };

        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::from(
                state
                    .encryption
                    .encrypt(
                        &serde_json::to_vec(&EncryptedRequest {
                            id: Some(RequestId {
                                session_id,
                                sequence_id: Some(0),
                            }),
                            method: "POST".into(),
                            uri: "/post?k=v".into(),
                            headers: HashMap::from([(
                                "Content-Type".into(),
                                "application/json".into(),
                            )]),
                            body: data_encoding::BASE64.encode(r#"{"key": "value"}"#.as_bytes()),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();

        let (res, body) = app.oneshot(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(
            res.status,
            StatusCode::OK,
            "Expected status: {:?}: {}",
            res.status,
            String::from_utf8(body.to_vec()).unwrap()
        );
        let body = state.encryption.decrypt(&body[..]).unwrap();
        let encrypted_response: EncryptedResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            encrypted_response.headers,
            HashMap::from([("content-type".into(), "application/json".into(),)])
        );
        let body = data_encoding::BASE64
            .decode(encrypted_response.body.as_bytes())
            .unwrap();
        assert_eq!(String::from_utf8(body).unwrap(), r#"{"key": "value"}"#);

        drop(session_stream);
    }

    #[tokio::test]
    async fn test_post_session_not_found() {
        let (app, state) = build_app();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::from(
                state
                    .encryption
                    .encrypt(
                        &serde_json::to_vec(&EncryptedRequest {
                            id: Some(RequestId {
                                session_id: "00000000-0000-0000-0000-000000000000".into(),
                                sequence_id: None,
                            }),
                            method: "POST".into(),
                            uri: "/post?k=v".into(),
                            headers: HashMap::from([(
                                "Content-Type".into(),
                                "application/json".into(),
                            )]),
                            body: data_encoding::BASE64.encode(r#"{"key": "value"}"#.as_bytes()),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();

        let (res, body) = app.oneshot(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(res.status, StatusCode::FORBIDDEN);
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "session not found"
        );
    }

    #[tokio::test]
    async fn test_post_session_request_replayed() {
        let (mut app, state) = build_app();
        let app = app.ready().await.unwrap();

        let mut session_stream = state.sessions.create_session();

        let session_id = match session_stream.next().await.unwrap() {
            SessionMessage::Start { session_id } => session_id,
            _ => panic!("expected start"),
        };

        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::from(
                state
                    .encryption
                    .encrypt(
                        &serde_json::to_vec(&EncryptedRequest {
                            id: Some(RequestId {
                                session_id: session_id.clone(),
                                sequence_id: Some(0),
                            }),
                            method: "POST".into(),
                            uri: "/post?k=v".into(),
                            headers: HashMap::from([(
                                "Content-Type".into(),
                                "application/json".into(),
                            )]),
                            body: data_encoding::BASE64.encode(r#"{"key": "value"}"#.as_bytes()),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();

        let (res, _) = app.call(req).await.unwrap().into_parts();
        assert_eq!(res.status, StatusCode::OK);

        let req = Request::builder()
            .method(Method::POST)
            .uri("/")
            .body(Body::from(
                state
                    .encryption
                    .encrypt(
                        &serde_json::to_vec(&EncryptedRequest {
                            id: Some(RequestId {
                                session_id: session_id.clone(),
                                sequence_id: Some(0),
                            }),
                            method: "POST".into(),
                            uri: "/post?k=v".into(),
                            headers: HashMap::from([(
                                "Content-Type".into(),
                                "application/json".into(),
                            )]),
                            body: data_encoding::BASE64.encode(r#"{"key": "value"}"#.as_bytes()),
                        })
                        .unwrap(),
                    )
                    .unwrap(),
            ))
            .unwrap();

        let (res, body) = app.call(req).await.unwrap().into_parts();
        let body = body.collect().await.unwrap().to_bytes();
        assert_eq!(res.status, StatusCode::FORBIDDEN);
        assert_eq!(
            String::from_utf8(body.to_vec()).unwrap(),
            "request replayed"
        );

        drop(session_stream);
    }
}
