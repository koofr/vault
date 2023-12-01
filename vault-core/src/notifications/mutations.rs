use crate::{notifications::state::Notification, store};

pub fn show(state: &mut store::State, notify: &store::Notify, message: String) {
    log::debug!("NotificationsService show: {}", message);

    let id = state.notifications.next_id.next();

    let notification = Notification { id, message };

    state.notifications.notifications.insert(id, notification);

    notify(store::Event::Notifications);
}

pub fn remove(state: &mut store::State, notify: &store::Notify, id: u32) {
    state.notifications.notifications.remove(&id);

    notify(store::Event::Notifications);
}

pub fn remove_all(state: &mut store::State, notify: &store::Notify) {
    state.notifications.notifications.clear();

    notify(store::Event::Notifications);
}
