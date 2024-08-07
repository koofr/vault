use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::Notify;

struct Listener<State, Event, MutationState> {
    id: u32,
    callback: Box<dyn Fn(&mut State, Rc<Notify<Event>>, &mut MutationState) + Send + Sync>,
}

pub struct MutationEventEmitter<State, Event, MutationState, MutationEvent> {
    listeners: Arc<Mutex<HashMap<MutationEvent, Vec<Arc<Listener<State, Event, MutationState>>>>>>,
}

impl<State, Event, MutationState, MutationEvent>
    MutationEventEmitter<State, Event, MutationState, MutationEvent>
where
    State: Debug + Clone + Send + Sync + 'static,
    Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    MutationState: Debug + Clone + Default + 'static,
    MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn on(
        &self,
        id: u32,
        events: &[MutationEvent],
        callback: Box<dyn Fn(&mut State, Rc<Notify<Event>>, &mut MutationState) + Send + Sync>,
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

    pub fn emit(
        &self,
        event: &MutationEvent,
        state: &mut State,
        notify: Rc<Notify<Event>>,
        mutation_state: &mut MutationState,
    ) {
        // we need to clone the listeners because they can change while
        // emitting. event_listeners also needs to be a separate variable
        // otherwise self.listeners is not unlocked before calling callbacks
        let event_listeners = self.listeners.lock().unwrap().get(event).cloned();

        if let Some(event_listeners) = event_listeners {
            for listener in event_listeners {
                (listener.callback)(state, notify.clone(), mutation_state);
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
