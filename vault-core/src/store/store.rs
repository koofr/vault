use std::sync::{Arc, Mutex};

use super::{event::Event, event_emitter::EventEmitter, state::State};

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
        self.mutate_notify(|state, notify| {
            notify(event);

            f(state)
        })
    }

    pub fn mutate_notify<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut State, &mut dyn FnMut(Event)) -> R,
    {
        let mut events: Vec<Event> = Vec::with_capacity(1);

        let mut notify = |event: Event| {
            if !events.contains(&event) {
                events.push(event);
            }
        };

        let mut state = self.state.lock().unwrap();

        let res = f(&mut state, &mut notify);

        drop(state);

        for event in events {
            self.event_emitter.emit(event);
        }

        res
    }
}
