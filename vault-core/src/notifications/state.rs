use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Notification {
    pub id: u32,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct NotificationsState {
    pub notifications: HashMap<u32, Notification>,
    pub next_id: u32,
}
