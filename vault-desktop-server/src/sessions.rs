use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use thiserror::Error;

use drop_stream::DropStream;
use futures::{stream, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{callbacks::Callbacks, request_id::RequestId};

#[derive(Error, Debug, Clone)]
pub enum RequestSessionError {
    #[error("session not found")]
    SessionNotFound,
    #[error("request replayed")]
    RequestReplayed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SessionMessage {
    Start {
        #[serde(rename = "sessionId")]
        session_id: String,
    },
    Callback {
        #[serde(rename = "callbackId")]
        callback_id: String,
    },
}

pub struct Session {
    pub id: String,
    pub callbacks: Arc<Callbacks>,
    pub seen_sequences: HashSet<u64>,
}

impl Session {
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        let callbacks = Arc::new(Callbacks::new());
        let seen_sequences = HashSet::new();

        Self {
            id,
            callbacks,
            seen_sequences,
        }
    }

    fn verify_sequence_id(&mut self, sequence_id: u64) -> Result<(), RequestSessionError> {
        if self.seen_sequences.contains(&sequence_id) {
            Err(RequestSessionError::RequestReplayed)
        } else {
            self.seen_sequences.insert(sequence_id);

            Ok(())
        }
    }
}

pub struct Sessions {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_session(&self) -> impl Stream<Item = SessionMessage> {
        let session = Session::new();

        let session_id = session.id.clone();

        let callbacks_stream = session
            .callbacks
            .stream()
            .map(|callback_id| SessionMessage::Callback { callback_id });

        let start_session_id = session_id.clone();

        let session_stream = Box::pin(
            stream::once(async move {
                SessionMessage::Start {
                    session_id: start_session_id,
                }
            })
            .chain(callbacks_stream),
        );

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.clone(), session);

        let drop_sessions = self.sessions.clone();
        let drop_session_id = session_id.clone();

        DropStream::new(session_stream, move || {
            let drop_session_id = drop_session_id;

            drop_sessions.lock().unwrap().remove(&drop_session_id);
        })
    }

    pub fn verify_request_id(&self, request_id: &RequestId) -> Result<(), RequestSessionError> {
        let mut sessions = self.sessions.lock().unwrap();

        match sessions.get_mut(&request_id.session_id) {
            Some(session) => match request_id.sequence_id {
                Some(sequence_id) => session.verify_sequence_id(sequence_id),
                None => Ok(()),
            },
            None => Err(RequestSessionError::SessionNotFound),
        }
    }

    pub fn get_callbacks(
        &self,
        request_id: &RequestId,
    ) -> Result<Arc<Callbacks>, RequestSessionError> {
        let sessions = self.sessions.lock().unwrap();

        match sessions.get(&request_id.session_id) {
            Some(session) => Ok(session.callbacks.clone()),
            None => Err(RequestSessionError::SessionNotFound),
        }
    }
}
