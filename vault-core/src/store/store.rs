use std::sync::{Arc, Mutex};

use super::event::Event;
use super::event_emitter::EventEmitter;
use super::state::State;

pub struct Store {
    state: Arc<Mutex<State>>,
    event_emitter: EventEmitter<Event>,
}

impl Store {
    pub fn new(initial_state: State) -> Store {
        Store {
            state: Arc::new(Mutex::new(initial_state)),
            event_emitter: EventEmitter::new(),
        }
    }

    pub fn get_next_id(&self) -> u32 {
        self.event_emitter.get_next_id()
    }

    pub fn on(&self, id: u32, events: &[Event], callback: Box<dyn Fn() + Send + Sync>) {
        self.event_emitter.on(id, events, callback)
    }

    pub fn remove_listener(&self, id: u32) {
        self.event_emitter.remove_listener(id)
    }

    pub fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&State) -> R,
    {
        let state = self.state.lock().unwrap();

        f(&state)
    }

    pub fn mutate<F, R>(&self, event: Event, f: F) -> R
    where
        F: FnOnce(&mut State) -> R,
    {
        // TODO mutate mutate function should be passed a "controller" with
        // event method so that every mutation can decide which events it
        // triggered
        let res = self.mutate_state(f);

        self.notify(event);

        res
    }

    pub fn notify(&self, event: Event) {
        self.event_emitter.emit(event);
    }

    pub fn notify_multi(&self, events: Vec<Event>) {
        for event in events {
            self.notify(event);
        }
    }

    pub fn mutate_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut State) -> R,
    {
        let mut state = self.state.lock().unwrap();

        f(&mut state)
    }
}
