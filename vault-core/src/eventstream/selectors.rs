use crate::{store, types::RemoteFileId};

use super::state::{MountListener, MountListenerState};

pub fn select_mount_listener_by_file_id<'a>(
    state: &'a store::State,
    file_id: &RemoteFileId,
) -> Option<&'a MountListener> {
    state
        .eventstream
        .mount_listeners_by_remote_file_id
        .get(file_id)
        .and_then(|mount_listener_id| state.eventstream.mount_listeners.get(mount_listener_id))
}

pub fn select_mount_listener_by_file_id_mut<'a>(
    state: &'a mut store::State,
    file_id: &RemoteFileId,
) -> Option<&'a mut MountListener> {
    state
        .eventstream
        .mount_listeners_by_remote_file_id
        .get(file_id)
        .and_then(|mount_listener_id| state.eventstream.mount_listeners.get_mut(mount_listener_id))
}

pub fn select_mount_listener_registered_by_file_id(
    state: &store::State,
    file_id: &RemoteFileId,
) -> bool {
    select_mount_listener_by_file_id(state, file_id)
        .filter(|mount_listener| {
            matches!(mount_listener.state, MountListenerState::Registered { .. })
        })
        .is_some()
}
