use crate::store;

use super::state::Notification;

pub fn select_notifications<'a>(state: &'a store::State) -> Vec<&'a Notification> {
    let mut notifications: Vec<&'a Notification> =
        state.notifications.notifications.values().collect();

    notifications.sort_by_key(|notification| notification.id);

    notifications
}
