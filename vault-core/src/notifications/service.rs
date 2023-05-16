use std::sync::Arc;

use crate::store;

use super::state::Notification;

pub struct NotificationsService {
    store: Arc<store::Store>,
}

impl NotificationsService {
    pub fn new(store: Arc<store::Store>) -> Self {
        Self { store }
    }

    pub fn show(&self, message: String) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Notifications);

            let id = state.notifications.next_id;

            state.notifications.next_id += 1;

            let notification = Notification { id, message };

            state.notifications.notifications.insert(id, notification);
        });
    }

    pub fn remove(&self, id: u32) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Notifications);

            state.notifications.notifications.remove(&id);
        });
    }

    pub fn remove_all(&self) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Notifications);

            state.notifications.notifications.clear();
        });
    }
}
