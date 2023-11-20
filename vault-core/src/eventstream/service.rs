use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use crate::{
    auth,
    eventstream::state::MountListenerState,
    remote_files::selectors::get_file_id,
    runtime, store,
    types::{MountId, RemoteFileId, RemotePath},
};

use super::{
    state::{ConnectionState, MountListener},
    Event, Message, Request, WebSocketClient,
};

const RECONNECT_DURATION: Duration = Duration::from_secs(3);
const PING_INTERVAL: Duration = Duration::from_secs(30);

pub struct MountSubscription {
    pub(self) file_id: RemoteFileId,
    pub(self) listener_id: u32,
    pub(self) eventstream_service: Arc<EventStreamService>,
}

impl std::fmt::Debug for MountSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MountSubscription")
            .field("file_id", &self.file_id)
            .field("listener_id", &self.listener_id)
            .finish()
    }
}

impl PartialEq for MountSubscription {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id && self.listener_id == other.listener_id
    }
}

impl Drop for MountSubscription {
    fn drop(&mut self) {
        // this can be called from within a store mutation
        self.eventstream_service
            .remove_mount_subscription(&self.file_id, self.listener_id);
    }
}

pub struct EventStreamService {
    base_url: String,
    websocket_client: Arc<Box<dyn WebSocketClient + Send + Sync>>,
    auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,

    connection_state: Arc<Mutex<ConnectionState>>,
    next_mount_listener_id: Arc<Mutex<u32>>,
    mount_listeners: Arc<Mutex<HashMap<u32, MountListener>>>,
    mount_subscriptions: Arc<Mutex<HashMap<RemoteFileId, Weak<MountSubscription>>>>,
    ping_alive: Arc<Mutex<Option<Arc<()>>>>,
    reconnect_alive: Arc<Mutex<Option<Arc<()>>>>,
}

impl EventStreamService {
    pub fn new(
        base_url: String,
        websocket_client: Box<dyn WebSocketClient + Send + Sync>,
        auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> EventStreamService {
        let websocket_client = Arc::new(websocket_client);

        EventStreamService {
            base_url,
            websocket_client,
            auth_provider,
            store,
            runtime,

            connection_state: Arc::new(Mutex::new(ConnectionState::Initial)),
            next_mount_listener_id: Arc::new(Mutex::new(1)),
            mount_listeners: Arc::new(Mutex::new(HashMap::new())),
            mount_subscriptions: Arc::new(Mutex::new(HashMap::new())),
            ping_alive: Arc::new(Mutex::new(None)),
            reconnect_alive: Arc::new(Mutex::new(None)),
        }
    }

    pub fn connect(self: Arc<Self>) {
        match *self.connection_state.lock().unwrap() {
            ConnectionState::Connecting
            | ConnectionState::Authenticating
            | ConnectionState::Connected { .. } => return,
            ConnectionState::Initial
            | ConnectionState::Reconnecting
            | ConnectionState::Disconnected => {}
        }

        log::debug!("Eventstream connecting");

        let base_ws_url = self
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");

        let url = format!("{}/events?wsauth=true", base_ws_url);

        let on_open_self = Arc::downgrade(&self);
        let on_message_self = Arc::downgrade(&self);
        let on_close_self = Arc::downgrade(&self);

        *self.connection_state.lock().unwrap() = ConnectionState::Connecting;

        self.websocket_client.open(
            url,
            Box::new(move || {
                if let Some(on_open_self) = on_open_self.upgrade() {
                    on_open_self.websocket_on_open()
                }
            }),
            Box::new(move |data: String| {
                if let Some(on_message_self) = on_message_self.upgrade() {
                    on_message_self.websocket_on_message(data)
                }
            }),
            Box::new(move || {
                if let Some(on_close_self) = on_close_self.upgrade() {
                    on_close_self.websocket_on_close()
                }
            }),
        );
    }

    pub fn disconnect(&self) {
        log::debug!("Eventstream disconnect");

        self.websocket_client.close();

        *self.connection_state.lock().unwrap() = ConnectionState::Disconnected;

        *self.ping_alive.lock().unwrap() = None;

        *self.reconnect_alive.lock().unwrap() = None;
    }

    fn websocket_on_open(self: Arc<Self>) {
        log::debug!("Eventstream authenticating");

        let on_open_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            *on_open_self.connection_state.lock().unwrap() = ConnectionState::Authenticating;

            let authorization = match on_open_self.auth_provider.get_authorization(false).await {
                Ok(authorization) => authorization,
                _ => {
                    on_open_self.websocket_client.close();

                    return;
                }
            };

            on_open_self.websocket_client.send(
                serde_json::to_string(&Request::Auth {
                    authorization: authorization,
                })
                .unwrap(),
            );
        }));
    }

    fn websocket_on_message(self: Arc<Self>, data: String) {
        // log::debug!("Eventstream message: {}", data);

        match serde_json::from_str(&data) {
            Ok(message) => match message {
                Message::Authenticated => {
                    log::debug!("Eventstream connected");

                    *self.connection_state.lock().unwrap() = ConnectionState::Connected {
                        next_request_id: 1,
                        request_id_to_mount_listener_id: HashMap::new(),
                        listener_id_to_mount_listener_id: HashMap::new(),
                    };

                    let mut connection_state = self.connection_state.lock().unwrap();

                    self.register_mounts(&mut connection_state);

                    self.clone().start_pinger();
                }
                Message::Registered {
                    request_id,
                    listener_id,
                } => {
                    let mut connection_state = self.connection_state.lock().unwrap();

                    if let Some(request_id) = request_id {
                        self.handle_registered(&mut connection_state, request_id, listener_id);
                    }
                }
                Message::Deregistered { .. } => {}
                Message::Event { listener_id, event } => {
                    // event handler can cause a MountSubscription to get
                    // dropped which calls remove_mount_subscription and could
                    // cause a deadlock. we must not hold any locks when
                    // handle_event is called.
                    let mount_listener = match &*self.connection_state.lock().unwrap() {
                        ConnectionState::Connected {
                            next_request_id: _,
                            request_id_to_mount_listener_id: _,
                            listener_id_to_mount_listener_id,
                        } => listener_id_to_mount_listener_id.get(&listener_id).and_then(
                            |mount_listener_id| {
                                let mount_listeners = self.mount_listeners.lock().unwrap();

                                mount_listeners.get(&mount_listener_id).cloned()
                            },
                        ),
                        _ => None,
                    };

                    if let Some(mount_listener) = mount_listener {
                        self.handle_event(mount_listener, event);
                    }
                }
                Message::Unknown => {}
            },
            _ => {}
        }
    }

    fn websocket_on_close(self: Arc<Self>) {
        match *self.connection_state.lock().unwrap() {
            ConnectionState::Disconnected => {
                return;
            }
            _ => {}
        }

        log::debug!("Eventstream error");

        *self.connection_state.lock().unwrap() = ConnectionState::Reconnecting;

        let mut mount_listeners = self.mount_listeners.lock().unwrap();

        for mount_listener in mount_listeners.values_mut() {
            mount_listener.state = MountListenerState::Unregistered;
        }

        drop(mount_listeners);

        *self.ping_alive.lock().unwrap() = None;

        self.clone().start_reconnecter()
    }

    pub fn get_mount_subscription(
        self: Arc<Self>,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Arc<MountSubscription> {
        let file_id = get_file_id(mount_id, &path.to_lowercase());

        let mut mount_subscriptions = self.mount_subscriptions.lock().unwrap();

        if let Some(mount_subscription) =
            mount_subscriptions.get(&file_id).and_then(|x| x.upgrade())
        {
            return mount_subscription;
        }

        let listener_id = self.create_mount_subscription(mount_id, path);

        let mount_subscription = Arc::new(MountSubscription {
            file_id: file_id.clone(),
            listener_id,
            eventstream_service: self.clone(),
        });

        mount_subscriptions.insert(file_id, Arc::downgrade(&mount_subscription));

        mount_subscription
    }

    fn create_mount_subscription(&self, mount_id: &MountId, path: &RemotePath) -> u32 {
        let mount_listener_id = {
            let mut next_mount_listener_id = self.next_mount_listener_id.lock().unwrap();
            let mount_listener_id = *next_mount_listener_id;
            *next_mount_listener_id += 1;
            mount_listener_id
        };

        self.mount_listeners.lock().unwrap().insert(
            mount_listener_id,
            MountListener {
                id: mount_listener_id,
                mount_id: mount_id.to_owned(),
                path: path.to_owned(),
                state: MountListenerState::Unregistered,
            },
        );

        let mut connection_state = self.connection_state.lock().unwrap();
        let mut mount_listeners = self.mount_listeners.lock().unwrap();
        let mut mount_listener = mount_listeners.get_mut(&mount_listener_id).unwrap();

        self.register_mount(&mut connection_state, &mut mount_listener);

        mount_listener_id
    }

    fn register_mount(
        &self,
        connection_state: &mut ConnectionState,
        mount_listener: &mut MountListener,
    ) {
        match connection_state {
            ConnectionState::Connected {
                ref mut next_request_id,
                ref mut request_id_to_mount_listener_id,
                listener_id_to_mount_listener_id: _,
            } => match mount_listener.state {
                MountListenerState::Unregistered => {
                    let request_id = *next_request_id;
                    *next_request_id = *next_request_id + 1;

                    request_id_to_mount_listener_id.insert(request_id, mount_listener.id);

                    mount_listener.state = MountListenerState::Registering;

                    self.websocket_client.send(
                        serde_json::to_string(&Request::Register {
                            request_id: Some(request_id),
                            mount_id: mount_listener.mount_id.clone(),
                            path: mount_listener.path.clone(),
                        })
                        .unwrap(),
                    );
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn register_mounts(&self, connection_state: &mut ConnectionState) {
        let mut mount_listeners = self.mount_listeners.lock().unwrap();

        for (_, mount_listener) in mount_listeners.iter_mut() {
            self.register_mount(connection_state, mount_listener);
        }
    }

    fn handle_registered(
        &self,
        connection_state: &mut ConnectionState,
        request_id: u32,
        listener_id: i64,
    ) {
        match connection_state {
            ConnectionState::Connected {
                next_request_id: _,
                ref mut request_id_to_mount_listener_id,
                ref mut listener_id_to_mount_listener_id,
            } => {
                if let Some(mount_listener_id) = request_id_to_mount_listener_id.remove(&request_id)
                {
                    listener_id_to_mount_listener_id.insert(listener_id, mount_listener_id);

                    let mut mount_listeners = self.mount_listeners.lock().unwrap();

                    if let Some(mount_listener) = mount_listeners.get_mut(&mount_listener_id) {
                        mount_listener.state = MountListenerState::Registered { listener_id };
                    }
                }
            }
            _ => {}
        }
    }

    /// This can be called from within a store mutation (MountSubscription::drop).
    pub(self) fn remove_mount_subscription(&self, file_id: &RemoteFileId, listener_id: u32) {
        self.mount_subscriptions.lock().unwrap().remove(file_id);

        if let Some(mount_listener) = self.mount_listeners.lock().unwrap().remove(&listener_id) {
            let mut connection_state = self.connection_state.lock().unwrap();

            self.deregister_mount(&mut connection_state, mount_listener);
        }
    }

    fn deregister_mount(
        &self,
        connection_state: &mut ConnectionState,
        mount_listener: MountListener,
    ) {
        match connection_state {
            ConnectionState::Connected {
                next_request_id: _,
                request_id_to_mount_listener_id: _,
                ref mut listener_id_to_mount_listener_id,
            } => match mount_listener.state {
                MountListenerState::Registered { listener_id } => {
                    listener_id_to_mount_listener_id.remove(&listener_id);

                    self.websocket_client
                        .send(serde_json::to_string(&Request::Deregister { listener_id }).unwrap());
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_event(&self, mount_listener: MountListener, event: Event) {
        self.store
            .mutate(|state, _, mutation_state, mutation_notify| {
                mutation_state
                    .eventstream_events
                    .events
                    .push((mount_listener, event));

                mutation_notify(
                    store::MutationEvent::EventstreamEvents,
                    state,
                    mutation_state,
                );
            });
    }

    fn start_pinger(self: Arc<Self>) {
        let ping_alive = Arc::new(());
        let ping_alive_weak = Arc::downgrade(&ping_alive);
        *self.ping_alive.lock().unwrap() = Some(ping_alive);

        let pinger_runtime = Arc::downgrade(&self.runtime);
        let pinger_websocket_client = Arc::downgrade(&self.websocket_client);

        self.runtime.spawn(Box::pin(async move {
            loop {
                if ping_alive_weak.upgrade().is_none() {
                    break;
                }

                (match pinger_runtime.upgrade() {
                    Some(runtime) => runtime.sleep(PING_INTERVAL),
                    None => return,
                })
                .await;

                if ping_alive_weak.upgrade().is_none() {
                    return;
                }

                match pinger_websocket_client.upgrade() {
                    Some(websocket_client) => {
                        websocket_client.send(serde_json::to_string(&Request::Ping).unwrap())
                    }
                    None => return,
                }
            }
        }));
    }

    fn start_reconnecter(self: Arc<Self>) {
        let reconnect_alive = Arc::new(());
        let reconnect_alive_weak = Arc::downgrade(&reconnect_alive);
        *self.reconnect_alive.lock().unwrap() = Some(reconnect_alive);

        let reconnecter_runtime = Arc::downgrade(&self.runtime);
        let reconnecter_self = Arc::downgrade(&self);

        self.runtime.spawn(Box::pin(async move {
            (match reconnecter_runtime.upgrade() {
                Some(runtime) => runtime.sleep(RECONNECT_DURATION),
                None => return,
            })
            .await;

            if reconnect_alive_weak.upgrade().is_none() {
                return;
            }

            if let Some(reconnecter_self) = reconnecter_self.upgrade() {
                reconnecter_self.connect();
            }
        }));
    }
}
