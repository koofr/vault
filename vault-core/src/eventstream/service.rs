use std::sync::{Arc, Mutex};

use crate::{auth, runtime, store};

use super::{mutations, Message, Request, WebSocketClient};

pub struct EventStreamService {
    base_url: String,
    websocket_client: Arc<Box<dyn WebSocketClient + Send + Sync>>,
    auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,
    eventstream_subscription_id: u32,

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
    ) -> Arc<EventStreamService> {
        let websocket_client = Arc::new(websocket_client);

        let eventstream_subscription_id = store.get_next_id();

        let eventstream_service = Arc::new(EventStreamService {
            base_url,
            websocket_client,
            auth_provider,
            store: store.clone(),
            runtime,
            eventstream_subscription_id,

            ping_alive: Arc::new(Mutex::new(None)),
            reconnect_alive: Arc::new(Mutex::new(None)),
        });

        let handle_eventstream_subscription_eventstream_service =
            Arc::downgrade(&eventstream_service);

        store.on(
            eventstream_subscription_id,
            &[store::Event::Eventstream],
            Box::new(move |mutation_state, _| {
                if let Some(eventstream_service) =
                    handle_eventstream_subscription_eventstream_service.upgrade()
                {
                    eventstream_service
                        .clone()
                        .handle_eventstream_mutation(mutation_state);
                }
            }),
        );

        eventstream_service
    }

    pub fn get_events_url(&self) -> String {
        let base_ws_url = self
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");

        format!("{}/events?wsauth=true", base_ws_url)
    }

    pub fn connect(self: Arc<Self>) {
        if !self
            .store
            .mutate(|state, notify, _, _| mutations::connect(state, notify))
        {
            return;
        }

        log::debug!("Eventstream connecting");

        let url = self.get_events_url();

        let on_open_self = Arc::downgrade(&self);
        let on_message_self = Arc::downgrade(&self);
        let on_close_self = Arc::downgrade(&self);

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
        if !self
            .store
            .mutate(|state, notify, _, _| mutations::disconnect(state, notify))
        {
            return;
        }

        log::debug!("Eventstream disconnect");

        self.websocket_client.close();

        *self.ping_alive.lock().unwrap() = None;
        *self.reconnect_alive.lock().unwrap() = None;
    }

    fn websocket_on_open(self: Arc<Self>) {
        log::debug!("Eventstream authenticating");

        self.store.mutate(|state, notify, _, _| {
            mutations::handle_opened(state, notify);
        });

        let on_open_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            let authorization = match on_open_self.auth_provider.get_authorization(false).await {
                Ok(authorization) => authorization,
                _ => {
                    on_open_self.websocket_client.close();

                    return;
                }
            };

            on_open_self
                .websocket_client
                .send(serde_json::to_string(&Request::Auth { authorization }).unwrap());
        }));
    }

    fn websocket_on_message(self: Arc<Self>, data: String) {
        // log::debug!("Eventstream message: {}", data);

        match serde_json::from_str(&data) {
            Ok(message) => match message {
                Message::Authenticated => {
                    log::debug!("Eventstream connected");

                    self.store.mutate(|state, notify, mutation_state, _| {
                        mutations::handle_authenticated(state, notify, mutation_state);
                    });

                    self.clone().start_pinger();
                }
                Message::Registered {
                    request_id,
                    listener_id,
                } => {
                    if let Some(request_id) = request_id {
                        self.store.mutate(|state, notify, mutation_state, _| {
                            mutations::handle_registered(
                                state,
                                notify,
                                mutation_state,
                                request_id,
                                listener_id,
                            );
                        });
                    }
                }
                Message::Deregistered { .. } => {}
                Message::Event { listener_id, event } => {
                    self.store
                        .mutate(|state, _, mutation_state, mutation_notify| {
                            mutations::handle_event(
                                state,
                                mutation_state,
                                mutation_notify,
                                listener_id,
                                event,
                            );
                        });
                }
                Message::Unknown => {}
            },
            _ => {}
        }
    }

    fn websocket_on_close(self: Arc<Self>) {
        if !self
            .store
            .mutate(|state, notify, _, _| mutations::handle_closed(state, notify))
        {
            return;
        }

        log::debug!("Eventstream error");

        *self.ping_alive.lock().unwrap() = None;

        self.clone().start_reconnecter()
    }

    pub fn handle_eventstream_mutation(self: Arc<Self>, mutation_state: &store::MutationState) {
        for request in mutation_state.eventstream.requests.iter() {
            let send_self = self.clone();
            let send_request = request.clone();

            self.runtime.spawn(Box::pin(async move {
                send_self
                    .websocket_client
                    .send(serde_json::to_string(&send_request).unwrap());
            }));
        }
    }

    fn start_pinger(self: Arc<Self>) {
        let ping_interval = self
            .store
            .with_state(|state| state.config.eventstream.ping_interval);

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
                    Some(runtime) => runtime.sleep(ping_interval),
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
        let reconnect_duration = self
            .store
            .with_state(|state| state.config.eventstream.reconnect_duration);

        let reconnect_alive = Arc::new(());
        let reconnect_alive_weak = Arc::downgrade(&reconnect_alive);
        *self.reconnect_alive.lock().unwrap() = Some(reconnect_alive);

        let reconnecter_runtime = Arc::downgrade(&self.runtime);
        let reconnecter_self = Arc::downgrade(&self);

        self.runtime.spawn(Box::pin(async move {
            (match reconnecter_runtime.upgrade() {
                Some(runtime) => runtime.sleep(reconnect_duration),
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

impl Drop for EventStreamService {
    fn drop(&mut self) {
        self.store.remove_listener(self.eventstream_subscription_id)
    }
}
