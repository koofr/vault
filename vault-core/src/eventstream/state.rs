use std::collections::{HashMap, HashSet};

use vault_store::NextId;

use crate::types::{MountId, RemoteFileId, RemotePath};

use super::{Event, Request};

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Initial,
    Connecting,
    Authenticating,
    Reconnecting,
    Connected {
        next_request_id: NextId,
        request_id_to_mount_listener_id: HashMap<u32, u32>,
        listener_id_to_mount_listener_id: HashMap<i64, u32>,
    },
    Disconnected,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Initial
    }
}

#[derive(Debug, Clone)]
pub enum MountListenerState {
    Unregistered,
    Registering { canceled: bool },
    Registered { listener_id: i64 },
}

#[derive(Debug, Clone)]
pub struct MountListener {
    pub id: u32,
    pub file_id: RemoteFileId,
    pub mount_id: MountId,
    pub path: RemotePath,
    pub state: MountListenerState,
    pub subscribers: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct EventstreamState {
    pub connection_state: ConnectionState,
    pub mount_listeners: HashMap<u32, MountListener>,
    pub mount_listeners_by_remote_file_id: HashMap<RemoteFileId, u32>,
    pub next_mount_listener_id: NextId,
}

impl EventstreamState {
    pub fn reset(&mut self) {
        *self = Self {
            next_mount_listener_id: self.next_mount_listener_id.clone(),
            ..Default::default()
        };
    }
}

#[derive(Debug, Clone, Default)]
pub struct EventstreamMutationState {
    pub requests: Vec<Request>,
}

#[derive(Debug, Clone, Default)]
pub struct EventstreamEventsMutationState {
    pub events: Vec<(MountListener, Event)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MountSubscription {
    pub file_id: RemoteFileId,
    pub subscriber: String,
}
