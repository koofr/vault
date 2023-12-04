use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use axum::response::{IntoResponse, Response};
use futures::{
    future::{BoxFuture, Shared},
    FutureExt,
};
use http::StatusCode;
use tokio::sync::oneshot::{self, Sender};

use super::{
    app_state::AppState,
    context::Context,
    interceptor::{Interceptor, InterceptorResult},
};

#[derive(Debug)]
pub struct DebugInterceptorPause {
    pub sender: Option<Sender<()>>,
    pub future: Shared<BoxFuture<'static, ()>>,
}

impl DebugInterceptorPause {
    pub fn new() -> Self {
        let (sender, receiver) = oneshot::channel();
        let future = receiver.map(|_| ()).boxed().shared();

        Self {
            sender: Some(sender),
            future,
        }
    }
}

impl Drop for DebugInterceptorPause {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}

#[derive(Debug, Default)]
pub struct DebugInterceptorQueue {
    pub requests: VecDeque<(http::Method, http::Uri, Sender<Option<Response>>)>,
}

impl Drop for DebugInterceptorQueue {
    fn drop(&mut self) {
        for (_, _, sender) in self.requests.drain(..) {
            let _ = sender.send(None);
        }
    }
}

#[derive(Debug, Default)]
pub struct DebugInterceptorState {
    pub queue: Option<DebugInterceptorQueue>,
    pub pause: Option<DebugInterceptorPause>,
    pub downloads_pause: Option<DebugInterceptorPause>,
    pub uploads_pause: Option<DebugInterceptorPause>,
}

#[derive(serde::Serialize)]
pub struct DebugInterceptorQueueRequestInfo {
    pub method: String,
    pub url: String,
}

#[derive(serde::Serialize)]
pub struct DebugInterceptorStateInfo {
    #[serde(rename = "queueEnabled")]
    pub queue_enabled: bool,
    #[serde(rename = "queueRequests")]
    pub queue_requests: Vec<DebugInterceptorQueueRequestInfo>,
    #[serde(rename = "pauseEnabled")]
    pub pause_enabled: bool,
    #[serde(rename = "downloadsPauseEnabled")]
    pub downloads_pause_enabled: bool,
    #[serde(rename = "uploadsPauseEnabled")]
    pub uploads_pause_enabled: bool,
}

fn handle_queue(
    parts: &http::request::Parts,
    state: &RwLock<DebugInterceptorState>,
    get_queue: impl Fn(&mut DebugInterceptorState) -> &mut Option<DebugInterceptorQueue>,
) -> Option<InterceptorResult> {
    let mut state = state.write().unwrap();

    match get_queue(&mut state) {
        Some(ref mut queue) => {
            let (sender, receiver) = oneshot::channel();

            queue
                .requests
                .push_back((parts.method.clone(), parts.uri.clone(), sender));

            Some(InterceptorResult::AsyncResponse(
                receiver
                    .map(|res| match res {
                        Ok(res) => res,
                        Err(_) => None,
                    })
                    .boxed(),
            ))
        }
        None => None,
    }
}

fn handle_pause(
    state: &RwLock<DebugInterceptorState>,
    get_pause: impl Fn(&DebugInterceptorState) -> &Option<DebugInterceptorPause>,
) -> Option<InterceptorResult> {
    let state = state.read().unwrap();

    get_pause(&state)
        .as_ref()
        .map(|pause| pause.future.clone())
        .map(|future| InterceptorResult::AsyncResponse(future.map(|_| None).boxed()))
}

pub fn get_debug_interceptor(
    state: Arc<RwLock<DebugInterceptorState>>,
    app_state: AppState,
    reset: Box<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync + 'static>,
) -> Interceptor {
    let reset = Arc::new(reset);

    Box::new(move |parts| {
        if parts.method == http::Method::OPTIONS {
            return InterceptorResult::Ignore;
        }

        let path = parts.uri.path();

        match path {
            "/debug/queue/enable" => {
                state
                    .write()
                    .unwrap()
                    .queue
                    .get_or_insert_with(|| Default::default());

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }
            "/debug/queue/disable" => {
                state.write().unwrap().queue.take();

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }
            "/debug/queue/next" => {
                if let Some(ref mut queue) = state.write().unwrap().queue {
                    if let Some((method, uri, sender)) = queue.requests.pop_front() {
                        let response = parts
                            .uri
                            .query()
                            .and_then(|query| {
                                url::form_urlencoded::parse(query.as_bytes())
                                    .into_owned()
                                    .find_map(|v| {
                                        if v.0 == "status" {
                                            Some(v.1.parse::<u16>().unwrap())
                                        } else {
                                            None
                                        }
                                    })
                            })
                            .map(|status_code| {
                                StatusCode::from_u16(status_code).unwrap().into_response()
                            });

                        let _ = sender.send(response);

                        return InterceptorResult::Response(
                            (StatusCode::OK, format!("ok\n\n{} {}", method, uri)).into_response(),
                        );
                    }
                }

                return InterceptorResult::Response(
                    (StatusCode::OK, "queue empty").into_response(),
                );
            }

            "/debug/pause" => {
                state.write().unwrap().pause = Some(DebugInterceptorPause::new());

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }
            "/debug/resume" => {
                state.write().unwrap().pause.take();

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }

            "/debug/downloads/pause" => {
                state.write().unwrap().downloads_pause = Some(DebugInterceptorPause::new());

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }
            "/debug/downloads/resume" => {
                state.write().unwrap().downloads_pause.take();

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }

            "/debug/uploads/pause" => {
                state.write().unwrap().uploads_pause = Some(DebugInterceptorPause::new());

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }
            "/debug/uploads/resume" => {
                state.write().unwrap().uploads_pause.take();

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }

            "/debug/reset" => {
                {
                    let mut state = state.write().unwrap();

                    state.queue.take();
                    state.pause.take();
                    state.downloads_pause.take();
                    state.uploads_pause.take();
                }

                let reset = reset.clone();

                return InterceptorResult::AsyncResponse(
                    async move {
                        reset().await;

                        Some((StatusCode::OK, "ok").into_response())
                    }
                    .boxed(),
                );
            }

            "/debug/oauth2/revoke" => {
                let mut state = app_state.state.write().unwrap();
                state.oauth2_access_tokens.clear();
                state.oauth2_refresh_tokens.clear();
                state.oauth2_codes.clear();

                return InterceptorResult::Response((StatusCode::OK, "ok").into_response());
            }

            "/debug/vault/repos/create" => {
                let app_state = app_state.clone();

                return InterceptorResult::AsyncResponse(
                    async move {
                        let user_id = app_state
                            .state
                            .read()
                            .unwrap()
                            .default_user_id
                            .clone()
                            .unwrap();

                        let context = Context {
                            user_id,
                            user_agent: None,
                        };

                        let repo = app_state
                            .vault_repos_create_service
                            .create_test_vault_repo(&context)
                            .await
                            .unwrap();

                        Some((StatusCode::OK, repo.id.0).into_response())
                    }
                    .boxed(),
                );
            }

            "/debug/state.json" => {
                let state = state.read().unwrap();

                return InterceptorResult::Response(
                    axum::Json(&DebugInterceptorStateInfo {
                        queue_enabled: state.queue.is_some(),
                        queue_requests: state
                            .queue
                            .as_ref()
                            .map(|queue| {
                                queue
                                    .requests
                                    .iter()
                                    .map(|(method, uri, _)| DebugInterceptorQueueRequestInfo {
                                        method: method.to_string(),
                                        url: uri.path_and_query().unwrap().to_string(),
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        pause_enabled: state.pause.is_some(),
                        downloads_pause_enabled: state.downloads_pause.is_some(),
                        uploads_pause_enabled: state.uploads_pause.is_some(),
                    })
                    .into_response(),
                );
            }

            "/debug" => {
                state.write().unwrap().uploads_pause.take();

                return InterceptorResult::Response(
                    (
                        StatusCode::OK,
                        [(
                            http::header::CONTENT_TYPE,
                            http::header::HeaderValue::from_static(mime::TEXT_HTML.as_ref()),
                        )],
                        include_bytes!("debug.html"),
                    )
                        .into_response(),
                );
            }

            _ => {}
        }

        if let Some(res) = handle_queue(parts, &state, |state| &mut state.queue) {
            return res;
        }

        if let Some(res) = handle_pause(&state, |state| &state.pause) {
            return res;
        }

        if path.contains("/content/api/") && path.contains("/files/get") {
            if let Some(res) = handle_pause(&state, |state| &state.downloads_pause) {
                return res;
            }
        }

        if path.contains("/content/api/") && path.contains("/files/put") {
            if let Some(res) = handle_pause(&state, |state| &state.uploads_pause) {
                return res;
            }
        }

        InterceptorResult::Ignore
    })
}
