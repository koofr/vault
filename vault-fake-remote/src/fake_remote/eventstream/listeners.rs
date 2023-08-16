use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tokio::sync::mpsc;

use crate::fake_remote::files::Path;

use super::{
    event::{event_relative_to, event_subject_id},
    subject::Subject,
    Event, Message,
};

#[derive(Debug, Clone)]
pub struct Listener {
    pub id: i64,
    pub subject: Subject,
    pub sender: mpsc::Sender<Message>,
}

impl Listener {
    pub fn new(id: i64, subject: Subject, sender: mpsc::Sender<Message>) -> Self {
        Self {
            id,
            subject,
            sender,
        }
    }

    pub async fn send(&self, message: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(message).await
    }

    pub async fn send_event(&self, event: Event) -> Result<(), mpsc::error::SendError<Message>> {
        self.send(Message::Event {
            listener_id: self.id,
            event,
        })
        .await
    }

    pub async fn send_registered(
        &self,
        request_id: u32,
    ) -> Result<(), mpsc::error::SendError<Message>> {
        self.send(Message::Registered {
            request_id: Some(request_id),
            listener_id: self.id,
        })
        .await
    }

    pub async fn send_deregistered(&self) -> Result<(), mpsc::error::SendError<Message>> {
        self.send(Message::Deregistered {
            listener_id: self.id,
        })
        .await
    }
}

#[derive(Debug)]
struct ListenersState {
    listeners_by_subjects: HashMap<String, HashMap<i64, Listener>>,
    listeners_to_subjects: HashMap<i64, String>,
    next_id: i64,
}

pub struct Listeners {
    state: Arc<RwLock<ListenersState>>,
}

impl Listeners {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ListenersState {
                listeners_by_subjects: HashMap::new(),
                listeners_to_subjects: HashMap::new(),
                next_id: 1,
            })),
        }
    }

    pub fn register(&self, subject: Subject, sender: mpsc::Sender<Message>) -> i64 {
        let mut state = self.state.write().unwrap();

        let listener_id = state.next_id;

        state.next_id += 1;

        let subject_id = subject.id();

        let entry = state.listeners_by_subjects.entry(subject_id.clone());

        let listeners = entry.or_insert_with(|| HashMap::new());

        listeners.insert(listener_id, Listener::new(listener_id, subject, sender));

        state.listeners_to_subjects.insert(listener_id, subject_id);

        listener_id
    }

    pub fn deregister(&self, listener_ids: &[i64]) {
        if listener_ids.is_empty() {
            return;
        }

        let mut state = self.state.write().unwrap();

        for listener_id in listener_ids {
            if let Some(subject_id) = state.listeners_to_subjects.remove(&listener_id) {
                let is_empty = if let Some(ref mut listeners) =
                    state.listeners_by_subjects.get_mut(&subject_id)
                {
                    listeners.remove(&listener_id);

                    listeners.is_empty()
                } else {
                    false
                };

                if is_empty {
                    state.listeners_by_subjects.remove(&subject_id);
                }
            }
        }
    }

    pub async fn process_event(&self, event: Event) {
        let listeners = if let Some(subject_id) = event_subject_id(&event) {
            let state = self.state.read().unwrap();

            if let Some(listeners) = state.listeners_by_subjects.get(subject_id) {
                listeners.values().cloned().collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        for listener in listeners {
            match &listener.subject {
                Subject::Mount { path, .. } => {
                    if let Some(event) = event_relative_to(event.clone(), &Path(path.to_owned())) {
                        if let Err(err) = listener.send_event(event).await {
                            log::info!("Eventstream failed to send event: {:?}", err);
                        }
                    }
                }
            }
        }
    }
}
