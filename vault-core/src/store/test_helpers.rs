use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use super::{Event, MutationNotify, MutationState, Notify, State, Store};

pub fn mutation() -> (Rc<Notify>, MutationState, Rc<MutationNotify>) {
    let notify: Rc<Notify> = Rc::new(Box::new(|_| {}));
    let mutation_state = MutationState::default();
    let mutation_notify: Rc<MutationNotify> = Rc::new(Box::new(move |_, _, _| {}));

    (notify, mutation_state, mutation_notify)
}

pub struct StoreWatcher {
    store: Arc<Store>,
    id: u32,
}

impl StoreWatcher {
    pub fn watch_store<Callback: Fn(&Store, usize) + Send + Sync + 'static>(
        store: Arc<Store>,
        events: &[Event],
        callback: Callback,
    ) -> Self {
        let counter = Arc::new(AtomicUsize::new(0));

        let id = store.get_next_id();

        let callback = Arc::new(Box::new(callback));
        let callback_callback = callback.clone();
        let callback_store = store.clone();
        let callback_counter = counter.clone();

        store.clone().on(
            id,
            events,
            Box::new(move |_| {
                let i = callback_counter.fetch_add(1, Ordering::SeqCst);

                callback_callback(&callback_store, i);
            }),
        );

        let i = counter.fetch_add(1, Ordering::SeqCst);

        callback(&store, i);

        Self { store, id }
    }

    pub fn watch_state<Callback: Fn(&State, usize) + Send + Sync + 'static>(
        store: Arc<Store>,
        events: &[Event],
        callback: Callback,
    ) -> Self {
        Self::watch_store(store, events, move |store, i| {
            store.with_state(|state| callback(state, i));
        })
    }
}

impl Drop for StoreWatcher {
    fn drop(&mut self) {
        self.store.remove_listener(self.id);
    }
}

pub struct StateRecorder<T> {
    watcher: StoreWatcher,
    entries: Arc<Mutex<Vec<T>>>,
}

impl<TransformedState: Send + 'static> StateRecorder<TransformedState> {
    pub fn record<Transform: Fn(&State) -> TransformedState + Send + Sync + 'static>(
        store: Arc<Store>,
        events: &[Event],
        transform: Transform,
    ) -> Self {
        let entries = Arc::new(Mutex::new(Vec::new()));

        let callback_entries = entries.clone();

        let watcher = StoreWatcher::watch_state(store, events, move |state, _| {
            callback_entries.lock().unwrap().push(transform(state));
        });

        Self { watcher, entries }
    }

    pub fn collect(self) -> Vec<TransformedState> {
        drop(self.watcher);

        self.entries.lock().unwrap().drain(..).collect()
    }

    pub fn collect_enumerated(self) -> Vec<(usize, TransformedState)> {
        self.collect().into_iter().enumerate().collect()
    }
}
