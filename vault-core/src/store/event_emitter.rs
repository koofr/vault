use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
};

pub struct Listener {
    id: u32,
    callback: Box<dyn Fn() + Send + Sync>,
}

pub struct EventEmitter<E> {
    listeners: Arc<Mutex<HashMap<E, Vec<Arc<Listener>>>>>,
    next_id: Arc<Mutex<u32>>,
}

impl<E> EventEmitter<E>
where
    E: Eq + Hash + Clone + std::fmt::Debug,
{
    pub fn new() -> EventEmitter<E> {
        EventEmitter {
            listeners: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_next_id(&self) -> u32 {
        let mut next_id = self.next_id.lock().unwrap();

        let id = *next_id + 1;

        *next_id = *next_id + 1;

        id
    }

    pub fn on(&self, id: u32, events: &[E], callback: Box<dyn Fn() + Send + Sync>) {
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

    pub fn emit(&self, event: E) {
        // We need to clone the listeners because they can change while
        // emitting. event_listeners also needs to be a separate variable
        // otherwise self.listeners is not unlocked before calling callbacks
        let event_listeners = self.listeners.lock().unwrap().get(&event).cloned();

        if let Some(event_listeners) = event_listeners {
            for listener in event_listeners {
                (listener.callback)();
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
