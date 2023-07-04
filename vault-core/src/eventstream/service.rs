use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use crate::{
    auth,
    remote_files::{selectors::get_file_id, RemoteFilesService},
    runtime,
    utils::path_utils::join_paths,
};

use super::{Event, Message, Request, WebSocketClient};

const RECONNECT_DURATION: Duration = Duration::from_secs(3);
const PING_INTERVAL: Duration = Duration::from_secs(30);

pub struct MountSubscription {
    pub(self) file_id: String,
    pub(self) listener_id: u32,
    pub(self) eventstream_service: Arc<EventStreamService>,
}

impl Drop for MountSubscription {
    fn drop(&mut self) {
        self.eventstream_service
            .remove_mount_subscription(&self.file_id, self.listener_id);
    }
}

enum ConnectionState {
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

enum MountListenerState {
    Unregistered,
    Registering,
    Registered { listener_id: i64 },
}

struct MountListener {
    id: u32,
    mount_id: String,
    path: String,
    state: MountListenerState,
}

pub struct EventStreamService {
    base_url: String,
    websocket_client: Box<dyn WebSocketClient + Send + Sync>,
    auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,
    remote_files_service: Arc<RemoteFilesService>,
    runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,

    connection_state: Arc<Mutex<ConnectionState>>,
    next_mount_listener_id: Arc<Mutex<u32>>,
    mount_listeners: Arc<Mutex<HashMap<u32, MountListener>>>,
    mount_subscriptions: Arc<Mutex<HashMap<String, Weak<MountSubscription>>>>,
    ping_alive: Arc<Mutex<Option<Arc<()>>>>,
    reconnect_alive: Arc<Mutex<Option<Arc<()>>>>,
}

impl EventStreamService {
    pub fn new(
        base_url: String,
        websocket_client: Box<dyn WebSocketClient + Send + Sync>,
        auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,
        remote_files_service: Arc<RemoteFilesService>,
        runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
    ) -> EventStreamService {
        EventStreamService {
            base_url,
            websocket_client,
            auth_provider,
            remote_files_service,
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

        let on_open_self = self.clone();
        let on_message_self = self.clone();
        let on_close_self = self.clone();

        *self.connection_state.lock().unwrap() = ConnectionState::Connecting;

        self.websocket_client.open(
            url,
            Box::new(move || on_open_self.clone().websocket_on_open()),
            Box::new(move |data: String| on_message_self.clone().websocket_on_message(data)),
            Box::new(move || on_close_self.clone().websocket_on_close()),
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
                    authorization: &authorization,
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

                    self.handle_registered(&mut connection_state, request_id, listener_id);
                }
                Message::Deregistered { .. } => {}
                Message::Event { listener_id, event } => {
                    let connection_state = self.connection_state.lock().unwrap();

                    self.handle_event(&connection_state, listener_id, event);
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
        mount_id: &str,
        path: &str,
    ) -> Arc<MountSubscription> {
        let file_id = get_file_id(mount_id, path);

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

    fn create_mount_subscription(&self, mount_id: &str, path: &str) -> u32 {
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
                            request_id,
                            mount_id: &mount_listener.mount_id,
                            path: &mount_listener.path,
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

    pub(self) fn remove_mount_subscription(&self, file_id: &str, listener_id: u32) {
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

    fn handle_event(&self, connection_state: &ConnectionState, listener_id: i64, event: Event) {
        match connection_state {
            ConnectionState::Connected {
                next_request_id: _,
                request_id_to_mount_listener_id: _,
                listener_id_to_mount_listener_id,
            } => {
                let mount_listeners = self.mount_listeners.lock().unwrap();

                if let Some(mount_listener) = listener_id_to_mount_listener_id
                    .get(&listener_id)
                    .and_then(|mount_listener_id| mount_listeners.get(&mount_listener_id))
                {
                    match event {
                        Event::FileCreatedEvent {
                            mount_id,
                            path,
                            file,
                            ..
                        } => {
                            self.remote_files_service.file_created(
                                &mount_id,
                                &join_paths(&mount_listener.path, &path),
                                file,
                            );
                        }
                        Event::FileRemovedEvent { mount_id, path, .. } => {
                            self.remote_files_service
                                .file_removed(&mount_id, &join_paths(&mount_listener.path, &path));
                        }
                        Event::FileCopiedEvent {
                            mount_id,
                            new_path,
                            file,
                            ..
                        } => {
                            self.remote_files_service.file_copied(
                                &mount_id,
                                &join_paths(&mount_listener.path, &new_path),
                                file,
                            );
                        }
                        Event::FileMovedEvent {
                            mount_id,
                            path,
                            new_path,
                            file,
                            ..
                        } => {
                            self.remote_files_service.file_moved(
                                &mount_id,
                                &join_paths(&mount_listener.path, &path),
                                &join_paths(&mount_listener.path, &new_path),
                                file,
                            );
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn start_pinger(self: Arc<Self>) {
        let ping_alive = Arc::new(());
        let ping_alive_weak = Arc::downgrade(&ping_alive);

        *self.ping_alive.lock().unwrap() = Some(ping_alive);

        let pinger_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            loop {
                if ping_alive_weak.upgrade().is_none() {
                    break;
                }

                pinger_self.runtime.sleep(PING_INTERVAL).await;

                if ping_alive_weak.upgrade().is_none() {
                    break;
                }

                pinger_self
                    .websocket_client
                    .send(serde_json::to_string(&Request::Ping).unwrap());
            }
        }));
    }

    fn start_reconnecter(self: Arc<Self>) {
        let reconnect_alive = Arc::new(());
        let reconnect_alive_weak = Arc::downgrade(&reconnect_alive);

        *self.reconnect_alive.lock().unwrap() = Some(reconnect_alive);

        let reconnecter_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            reconnecter_self.runtime.sleep(RECONNECT_DURATION).await;

            if reconnect_alive_weak.upgrade().is_some() {
                reconnecter_self.connect();
            }
        }));
    }
}
