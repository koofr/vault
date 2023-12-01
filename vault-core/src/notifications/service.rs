use std::{sync::Arc, time::Duration};

use crate::{runtime, store};

use super::mutations;

pub struct NotificationsService {
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,
}

impl NotificationsService {
    pub fn new(store: Arc<store::Store>, runtime: Arc<runtime::BoxRuntime>) -> Self {
        Self { store, runtime }
    }

    pub fn show(&self, message: String) {
        self.store.mutate(|state, notify, _, _| {
            mutations::show(state, notify, message);
        });
    }

    pub fn remove(&self, id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::remove(state, notify, id);
        });
    }

    pub async fn remove_after(&self, id: u32, duration: Duration) {
        self.runtime.sleep(duration).await;

        self.remove(id);
    }

    pub fn remove_all(&self) {
        self.store.mutate(|state, notify, _, _| {
            mutations::remove_all(state, notify);
        });
    }
}
