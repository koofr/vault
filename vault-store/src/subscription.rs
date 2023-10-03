use std::{
    collections::{hash_map, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};

use super::Store;

pub struct Subscription<State, Event, MutationState, MutationEvent> {
    store: Arc<Store<State, Event, MutationState, MutationEvent>>,
    cleanups: Arc<Mutex<HashMap<u32, Box<dyn Fn() + Send + Sync + 'static>>>>,
}

impl<State, Event, MutationState, MutationEvent>
    Subscription<State, Event, MutationState, MutationEvent>
where
    State: Debug + Clone + Send + Sync + 'static,
    Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    MutationState: Debug + Clone + Default + 'static,
    MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
{
    pub fn new(store: Arc<Store<State, Event, MutationState, MutationEvent>>) -> Self {
        Self {
            store,
            cleanups: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[Event],
        callback: Box<dyn Fn() + Send + Sync + 'static>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn() -> T + Send + Sync + 'static,
    ) -> u32 {
        self.subscribe_changed(events, callback, subscription_data, move |entry| {
            let new_data = generate_data();

            match entry {
                hash_map::Entry::Occupied(mut o) => {
                    if &new_data == o.get() {
                        false
                    } else {
                        o.insert(new_data);

                        true
                    }
                }
                hash_map::Entry::Vacant(v) => {
                    v.insert(new_data);

                    true
                }
            }
        })
    }

    pub fn subscribe_changed<T: Clone + Send + 'static>(
        &self,
        events: &[Event],
        callback: Box<dyn Fn() + Send + Sync + 'static>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(hash_map::Entry<'_, u32, T>) -> bool + Send + Sync + 'static,
    ) -> u32 {
        let id = self.store.get_next_id();

        let generate_data = Arc::new(generate_data);

        let callback = Arc::new(callback);
        let callback_subscription_data = subscription_data.clone();
        let callback_generate_data = generate_data.clone();

        self.store.on(
            id,
            events,
            Box::new(move |_, add_side_effect| {
                let changed =
                    callback_generate_data(callback_subscription_data.lock().unwrap().entry(id));

                if changed {
                    let side_effect_callback = callback.clone();

                    add_side_effect(Box::new(move || side_effect_callback()));
                }
            }),
        );

        let cleanup_subscription_data = subscription_data.clone();

        let cleanup = Box::new(move || {
            cleanup_subscription_data
                .clone()
                .lock()
                .unwrap()
                .remove(&id);
        });

        self.cleanups.lock().unwrap().insert(id, cleanup);

        let mut subscription_data = subscription_data.lock().unwrap();
        let _ = generate_data(subscription_data.entry(id));

        id
    }

    pub fn get_data<T: Clone + Send>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Option<T> {
        subscription_data.lock().unwrap().get(&id).cloned()
    }

    pub fn unsubscribe(&self, id: u32) {
        self.store.remove_listener(id);

        let cleanup = self.cleanups.lock().unwrap().remove(&id);

        if let Some(cleanup) = cleanup {
            cleanup();
        }
    }
}

pub fn update_if<T: Clone + Send + 'static, F: Fn() -> T, G: Fn(&T) -> bool>(
    entry: hash_map::Entry<'_, u32, T>,
    generate: F,
    should_regenerate: G,
) -> bool {
    match entry {
        hash_map::Entry::Occupied(mut o) => {
            if !should_regenerate(o.get()) {
                false
            } else {
                o.insert(generate());

                true
            }
        }
        hash_map::Entry::Vacant(v) => {
            v.insert(generate());

            true
        }
    }
}
