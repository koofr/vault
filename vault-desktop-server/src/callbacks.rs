use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use drop_stream::DropStream;
use futures::{Stream, channel::mpsc};
use serde::{Deserialize, Serialize};

use vault_web_api::web_vault_base::Callback;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallbackId(pub String);

pub struct Callbacks {
    senders: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<(String, u32)>>>>,
}

impl Callbacks {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn cb(&self, callback_id: CallbackId) -> Callback {
        let callbacks_senders = self.senders.clone();

        Box::new(move |subscription_id| {
            let senders = callbacks_senders.lock().unwrap();

            for sender in senders.values() {
                let _ = sender.unbounded_send((callback_id.0.clone(), subscription_id));
            }
        })
    }

    pub fn stream(&self) -> impl Stream<Item = (String, u32)> + use<> {
        let callbacks_senders = self.senders.clone();

        let (callbacks_sender, callbacks_receiver) = mpsc::unbounded();

        let id = uuid::Uuid::new_v4().to_string();

        callbacks_senders
            .lock()
            .unwrap()
            .insert(id.clone(), callbacks_sender);

        DropStream::new(callbacks_receiver, move || {
            let id = id;

            callbacks_senders.lock().unwrap().remove(&id);
        })
    }
}
