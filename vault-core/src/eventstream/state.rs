use std::collections::HashMap;

use crate::types::{MountId, RemotePath};

use super::Event;

pub enum ConnectionState {
    Initial,
    Connecting,
    Authenticating,
    Reconnecting,
    Connected {
        next_request_id: u32,
        request_id_to_mount_listener_id: HashMap<u32, u32>,
        listener_id_to_mount_listener_id: HashMap<i64, u32>,
    },
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum MountListenerState {
    Unregistered,
    Registering,
    Registered { listener_id: i64 },
}

#[derive(Debug, Clone)]
pub struct MountListener {
    pub id: u32,
    pub mount_id: MountId,
    pub path: RemotePath,
    pub state: MountListenerState,
}

#[derive(Debug, Clone, Default)]
pub struct EventstreamEventsMutationState {
    pub events: Vec<(MountListener, Event)>,
}
