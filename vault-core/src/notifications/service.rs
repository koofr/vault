use std::{sync::Arc, time::Duration};

use crate::{runtime, store};

use super::state::Notification;

pub struct NotificationsService {
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,
}

impl NotificationsService {
    pub fn new(store: Arc<store::Store>, runtime: Arc<runtime::BoxRuntime>) -> Self {
        Self { store, runtime }
    }

    pub fn show(&self, message: String) {
        log::debug!("NotificationsService show: {}", message);

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Notifications);

            let id = state.notifications.next_id.next();

            let notification = Notification { id, message };

            state.notifications.notifications.insert(id, notification);
        });
    }

    pub fn remove(&self, id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Notifications);

            state.notifications.notifications.remove(&id);
        });
    }

    pub async fn remove_after(&self, id: u32, duration: Duration) {
        self.runtime.sleep(duration).await;

        self.remove(id);
    }

    pub fn remove_all(&self) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Notifications);

            state.notifications.notifications.clear();
        });
    }
}
