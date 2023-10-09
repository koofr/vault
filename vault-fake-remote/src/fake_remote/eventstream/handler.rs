use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use http::StatusCode;
use tokio::sync::mpsc;

use crate::fake_remote::{
    errors::{ApiErrorCode, FakeRemoteError},
    extract::{
        get_authorization_access_token, get_user_id_by_access_token, ExtractEventstreamListeners,
        ExtractState,
    },
    files::Path,
    state::FakeRemoteState,
};

use super::{Listeners, Message, Request, Subject};

pub async fn eventstream(
    ExtractState(state): ExtractState,
    ExtractEventstreamListeners(listeners): ExtractEventstreamListeners,
    ws: WebSocketUpgrade,
) -> Result<Response, FakeRemoteError> {
    Ok(ws.on_upgrade(move |socket| async {
        let mut handler = EventstreamHandler::new(state, listeners, socket);

        match handler.handle().await {
            Ok(()) => {}
            Err(err) => {
                log::info!("eventstream error: {:?}", err);
            }
        }
    }))
}

struct EventstreamHandler {
    state: Arc<RwLock<FakeRemoteState>>,
    listeners: Arc<Listeners>,
    listener_ids: HashSet<i64>,
    socket_writer: SplitSink<WebSocket, WsMessage>,
    socket_reader: SplitStream<WebSocket>,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
}

impl EventstreamHandler {
    fn new(
        state: Arc<RwLock<FakeRemoteState>>,
        listeners: Arc<Listeners>,
        socket: WebSocket,
    ) -> Self {
        let listener_ids = HashSet::new();

        let (socket_writer, socket_reader) = socket.split();

        let (sender, receiver) = mpsc::channel::<Message>(100);

        Self {
            state,
            listeners,
            listener_ids,
            socket_writer,
            socket_reader,
            sender,
            receiver,
        }
    }

    async fn handle(&mut self) -> Result<(), FakeRemoteError> {
        loop {
            tokio::select! {
                res = self.socket_reader.next() => if !self.handle_socket_message(res).await? {
                    return Ok(());
                },
                res = self.receiver.recv() => self.handle_receiver_message(res).await?
            }
        }
    }

    async fn handle_socket_message(
        &mut self,
        res: Option<Result<WsMessage, axum::Error>>,
    ) -> Result<bool, FakeRemoteError> {
        match res {
            Some(Ok(message)) => match message {
                WsMessage::Text(text) => {
                    let request: Request = serde_json::from_str(&text).unwrap();

                    if let Some(message) = self.handle_request(&request).await? {
                        self.sender.send(message).await.map_err(|err| {
                            FakeRemoteError::ApiError(
                                StatusCode::INTERNAL_SERVER_ERROR,
                                ApiErrorCode::Other,
                                err.to_string(),
                                None,
                            )
                        })?;
                    }

                    Ok(true)
                }
                WsMessage::Close(_) => Ok(false),
                _ => Ok(true),
            },
            Some(Err(err)) => {
                log::info!("events proxy downstream read error: {:?}", err);

                Ok(false)
            }
            None => Ok(false),
        }
    }

    async fn handle_receiver_message(
        &mut self,
        res: Option<Message>,
    ) -> Result<(), FakeRemoteError> {
        if let Some(message) = res {
            self.socket_writer
                .send(WsMessage::Text(serde_json::to_string(&message).unwrap()))
                .await
                .map_err(|err| {
                    FakeRemoteError::ApiError(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiErrorCode::Other,
                        err.to_string(),
                        None,
                    )
                })?;
        }

        Ok(())
    }

    async fn handle_request<'a>(
        &mut self,
        request: &'a Request<'a>,
    ) -> Result<Option<Message>, FakeRemoteError> {
        match request {
            Request::Auth { authorization } => {
                let access_token = get_authorization_access_token(authorization)?;

                {
                    let state = self.state.read().unwrap();

                    let _ =
                        get_user_id_by_access_token(&state, access_token).map(str::to_string)?;
                }

                Ok(Some(Message::Authenticated))
            }
            Request::Register {
                request_id,
                mount_id,
                path,
            } => {
                let path = Path::from_str(path).map_err(|err| {
                    FakeRemoteError::ApiError(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiErrorCode::Other,
                        err.to_string(),
                        None,
                    )
                })?;

                let listener_id = self.listeners.register(
                    Subject::Mount {
                        id: mount_id.to_string(),
                        path: path.0,
                    },
                    self.sender.clone(),
                );

                self.listener_ids.insert(listener_id);

                Ok(Some(Message::Registered {
                    request_id: request_id.clone(),
                    listener_id,
                }))
            }
            Request::Deregister { listener_id } => {
                self.listeners.deregister(&[*listener_id]);

                self.listener_ids.remove(listener_id);

                Ok(Some(Message::Deregistered {
                    listener_id: listener_id.clone(),
                }))
            }
            Request::Ping => Ok(None),
            Request::Unknown => Err(FakeRemoteError::ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::Other,
                "Unknown message".into(),
                None,
            )),
        }
    }
}

impl Drop for EventstreamHandler {
    fn drop(&mut self) {
        let listener_ids: Vec<i64> = self.listener_ids.iter().cloned().collect();

        self.listeners.deregister(&listener_ids);
    }
}
