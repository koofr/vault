use std::collections::HashSet;

use crate::{
    remote_files, store,
    types::{MountId, RemotePath},
};

use super::{
    selectors,
    state::{ConnectionState, MountListener, MountListenerState, MountSubscription},
    Event, Request,
};

pub fn connect(state: &mut store::State, notify: &store::Notify) -> bool {
    match state.eventstream.connection_state {
        ConnectionState::Connecting
        | ConnectionState::Authenticating
        | ConnectionState::Connected { .. } => return false,
        ConnectionState::Initial
        | ConnectionState::Reconnecting
        | ConnectionState::Disconnected => {}
    }

    notify(store::Event::Eventstream);

    state.eventstream.connection_state = ConnectionState::Connecting;

    true
}

pub fn disconnect(state: &mut store::State, notify: &store::Notify) -> bool {
    if matches!(
        state.eventstream.connection_state,
        ConnectionState::Disconnected
    ) {
        return false;
    }

    notify(store::Event::Eventstream);

    state.eventstream.connection_state = ConnectionState::Disconnected;

    true
}

pub fn handle_opened(state: &mut store::State, notify: &store::Notify) {
    notify(store::Event::Eventstream);

    state.eventstream.connection_state = ConnectionState::Authenticating;
}

pub fn handle_closed(state: &mut store::State, notify: &store::Notify) -> bool {
    if matches!(
        state.eventstream.connection_state,
        ConnectionState::Disconnected
    ) {
        return false;
    }

    notify(store::Event::Eventstream);

    state.eventstream.connection_state = ConnectionState::Reconnecting;

    for mount_listener in state.eventstream.mount_listeners.values_mut() {
        mount_listener.state = MountListenerState::Unregistered;
    }

    true
}

pub fn handle_authenticated(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
) {
    notify(store::Event::Eventstream);

    state.eventstream.connection_state = ConnectionState::Connected {
        next_request_id: Default::default(),
        request_id_to_mount_listener_id: Default::default(),
        listener_id_to_mount_listener_id: Default::default(),
    };

    let mount_listeners = state
        .eventstream
        .mount_listeners
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    for mount_listener_id in mount_listeners {
        register_mount(state, notify, mutation_state, mount_listener_id);
    }
}

pub fn handle_registered(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    request_id: u32,
    listener_id: i64,
) {
    match state.eventstream.connection_state {
        ConnectionState::Connected {
            ref mut request_id_to_mount_listener_id,
            ref mut listener_id_to_mount_listener_id,
            ..
        } => {
            if let Some(mount_listener_id) = request_id_to_mount_listener_id.remove(&request_id) {
                notify(store::Event::Eventstream);

                if let Some(mount_listener) = state
                    .eventstream
                    .mount_listeners
                    .get_mut(&mount_listener_id)
                {
                    match mount_listener.state {
                        MountListenerState::Registering { canceled } => {
                            if canceled {
                                mutation_state
                                    .eventstream
                                    .requests
                                    .push(Request::Deregister { listener_id });
                            } else {
                                listener_id_to_mount_listener_id
                                    .insert(listener_id, mount_listener_id);

                                mount_listener.state =
                                    MountListenerState::Registered { listener_id };
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }
}

pub fn handle_event(
    state: &mut store::State,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    listener_id: i64,
    event: Event,
) {
    let mount_listener = match &state.eventstream.connection_state {
        ConnectionState::Connected {
            listener_id_to_mount_listener_id,
            ..
        } => listener_id_to_mount_listener_id
            .get(&listener_id)
            .and_then(|mount_listener_id| {
                state
                    .eventstream
                    .mount_listeners
                    .get(&mount_listener_id)
                    .cloned()
            }),
        _ => None,
    };

    if let Some(mount_listener) = mount_listener {
        mutation_state
            .eventstream_events
            .events
            .push((mount_listener, event));

        mutation_notify(
            store::MutationEvent::EventstreamEvents,
            state,
            mutation_state,
        );
    }
}

pub fn register_mount(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mount_listener_id: u32,
) {
    match state.eventstream.connection_state {
        ConnectionState::Connected {
            ref mut next_request_id,
            ref mut request_id_to_mount_listener_id,
            ..
        } => {
            if let Some(mount_listener) = state
                .eventstream
                .mount_listeners
                .get_mut(&mount_listener_id)
            {
                match mount_listener.state {
                    MountListenerState::Unregistered => {
                        notify(store::Event::Eventstream);

                        let request_id = next_request_id.next();

                        request_id_to_mount_listener_id.insert(request_id, mount_listener_id);

                        mount_listener.state = MountListenerState::Registering { canceled: false };

                        mutation_state.eventstream.requests.push(Request::Register {
                            request_id: Some(request_id),
                            mount_id: mount_listener.mount_id.clone(),
                            path: mount_listener.path.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

pub fn deregister_mount(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mount_listener_id: u32,
) {
    match state.eventstream.connection_state {
        ConnectionState::Connected {
            ref mut listener_id_to_mount_listener_id,
            ..
        } => {
            if let Some(mount_listener) = state
                .eventstream
                .mount_listeners
                .get_mut(&mount_listener_id)
            {
                match mount_listener.state {
                    MountListenerState::Registered { listener_id } => {
                        notify(store::Event::Eventstream);

                        listener_id_to_mount_listener_id.remove(&listener_id);

                        mutation_state
                            .eventstream
                            .requests
                            .push(Request::Deregister { listener_id });
                    }
                    MountListenerState::Registering { ref mut canceled } => {
                        notify(store::Event::Eventstream);

                        *canceled = true;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

pub fn add_mount_subscriber(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mount_id: MountId,
    path: RemotePath,
    subscriber: String,
) -> MountSubscription {
    let file_id = remote_files::selectors::get_file_id(&mount_id, &path.to_lowercase());

    let mount_subscription = MountSubscription {
        file_id: file_id.clone(),
        subscriber: subscriber.clone(),
    };

    if let Some(mount_listener) = selectors::select_mount_listener_by_file_id_mut(state, &file_id) {
        if mount_listener
            .subscribers
            .insert(mount_subscription.subscriber.clone())
        {
            notify(store::Event::Eventstream);
        }

        return mount_subscription;
    }

    notify(store::Event::Eventstream);

    let mount_listener_id = state.eventstream.next_mount_listener_id.next();

    let mount_listener = MountListener {
        id: mount_listener_id,
        file_id: file_id.clone(),
        mount_id: mount_id.clone(),
        path: path.clone(),
        state: MountListenerState::Unregistered,
        subscribers: HashSet::from([subscriber]),
    };

    state
        .eventstream
        .mount_listeners
        .insert(mount_listener_id, mount_listener);

    state
        .eventstream
        .mount_listeners_by_remote_file_id
        .insert(file_id, mount_listener_id);

    register_mount(state, notify, mutation_state, mount_listener_id);

    mount_subscription
}

pub fn remove_mount_subscriber(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mount_subscription: MountSubscription,
) {
    let MountSubscription {
        file_id,
        subscriber,
    } = mount_subscription;

    let mut remove_mount_listener_id: Option<u32> = None;

    if let Some(mount_listener) = selectors::select_mount_listener_by_file_id_mut(state, &file_id)
        .filter(|mount_listener| mount_listener.subscribers.contains(&subscriber))
    {
        if mount_listener.subscribers.remove(&subscriber) {
            notify(store::Event::Eventstream);
        }

        if mount_listener.subscribers.is_empty() {
            remove_mount_listener_id = Some(mount_listener.id);
        }
    }

    if let Some(mount_listener_id) = remove_mount_listener_id {
        deregister_mount(state, notify, mutation_state, mount_listener_id);

        if let Some(mount_listener) = state.eventstream.mount_listeners.remove(&mount_listener_id) {
            state
                .eventstream
                .mount_listeners_by_remote_file_id
                .remove(&mount_listener.file_id);
        }
    }
}
