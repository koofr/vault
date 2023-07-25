use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};

struct Listener<MutationState> {
    id: u32,
    callback: Box<dyn Fn(&MutationState) + Send + Sync>,
}

pub struct EventEmitter<Event, MutationState> {
    listeners: Arc<Mutex<HashMap<Event, Vec<Arc<Listener<MutationState>>>>>>,
}

impl<Event, MutationState> EventEmitter<Event, MutationState>
where
    Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    MutationState: Debug + Clone + Default + 'static,
{
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn on(
        &self,
        id: u32,
        events: &[Event],
        callback: Box<dyn Fn(&MutationState) + Send + Sync>,
    ) {
        let listener = Arc::new(Listener { id, callback });

        let mut listeners = self.listeners.lock().unwrap();

        for event in events {
            match listeners.get_mut(&event) {
                Some(callbacks) => {
                    callbacks.push(listener.clone());
                }
                None => {
                    listeners.insert(event.clone(), vec![listener.clone()]);
                }
            }
        }
    }

    pub fn emit(&self, event: &Event, mutation_state: &MutationState) {
        // we need to clone the listeners because they can change while
        // emitting. event_listeners also needs to be a separate variable
        // otherwise self.listeners is not unlocked before calling callbacks
        let event_listeners = self.listeners.lock().unwrap().get(event).cloned();

        if let Some(event_listeners) = event_listeners {
            for listener in event_listeners {
                (listener.callback)(mutation_state);
            }
        }
    }

    pub fn remove_listener(&self, id: u32) -> () {
        let mut listeners = self.listeners.lock().unwrap();

        for (_, event_listeners) in listeners.iter_mut() {
            // if `on` was called with multiple same events, `event_listeners`
            // will have multiple listeners with the same id, so we have to
            // remove all of them
            event_listeners.retain(|listener| listener.id != id);
        }
    }
}
