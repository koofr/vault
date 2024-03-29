use std::{
    fmt::Debug,
    hash::Hash,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use crate::{MutationNotify, Notify, RemoveListener, Store};

pub fn mutation<State, Event, MutationState, MutationEvent>() -> (
    Rc<Notify<Event>>,
    MutationState,
    Rc<MutationNotify<MutationEvent, State, MutationState>>,
)
where
    State: Debug + Clone + Send + Sync + 'static,
    Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    MutationState: Debug + Clone + Default + 'static,
    MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
{
    let notify: Rc<Notify<Event>> = Rc::new(Box::new(|_| {}));
    let mutation_state = MutationState::default();
    let mutation_notify: Rc<MutationNotify<MutationEvent, State, MutationState>> =
        Rc::new(Box::new(move |_, _, _| {}));

    (notify, mutation_state, mutation_notify)
}

pub struct StoreWatcher {
    _remove_listener: RemoveListener,
}

impl StoreWatcher {
    pub fn watch_store<
        State,
        Event,
        MutationState,
        MutationEvent,
        Callback: Fn(&Store<State, Event, MutationState, MutationEvent>, usize) + Send + Sync + 'static,
    >(
        store: Arc<Store<State, Event, MutationState, MutationEvent>>,
        events: &[Event],
        callback: Callback,
    ) -> Self
    where
        State: Debug + Clone + Send + Sync + 'static,
        Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
        MutationState: Debug + Clone + Default + 'static,
        MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    {
        let counter = Arc::new(AtomicUsize::new(0));

        let id = store.get_next_id();

        let callback = Arc::new(Box::new(callback));
        let callback_callback = callback.clone();
        let callback_store = store.clone();
        let callback_counter = counter.clone();

        store.clone().on(
            id,
            events,
            Box::new(move |_, _| {
                let i = callback_counter.fetch_add(1, Ordering::SeqCst);

                callback_callback(&callback_store, i);
            }),
        );

        let i = counter.fetch_add(1, Ordering::SeqCst);

        callback(&store, i);

        Self {
            _remove_listener: RemoveListener::new(store.clone(), id),
        }
    }

    pub fn watch_state<
        State,
        Event,
        MutationState,
        MutationEvent,
        Callback: Fn(&State, usize) + Send + Sync + 'static,
    >(
        store: Arc<Store<State, Event, MutationState, MutationEvent>>,
        events: &[Event],
        callback: Callback,
    ) -> Self
    where
        State: Debug + Clone + Send + Sync + 'static,
        Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
        MutationState: Debug + Clone + Default + 'static,
        MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    {
        Self::watch_store(store, events, move |store, i| {
            store.with_state(|state| callback(state, i));
        })
    }
}

pub struct StateRecorder<T> {
    watcher: StoreWatcher,
    entries: Arc<Mutex<Vec<T>>>,
}

impl<TransformedState: Send + 'static> StateRecorder<TransformedState> {
    pub fn record<
        State,
        Event,
        MutationState,
        MutationEvent,
        Transform: Fn(&State) -> TransformedState + Send + Sync + 'static,
    >(
        store: Arc<Store<State, Event, MutationState, MutationEvent>>,
        events: &[Event],
        transform: Transform,
    ) -> Self
    where
        State: Debug + Clone + Send + Sync + 'static,
        Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
        MutationState: Debug + Clone + Default + 'static,
        MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    {
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

    pub fn check_recorded(
        self,
        check_len: impl FnOnce(usize),
        check_entry: impl Fn(usize, TransformedState),
    ) {
        let entries = self.collect_enumerated();
        let entries_len = entries.len();

        for (i, entry) in entries {
            check_entry(i, entry);
        }

        // check len at the end so that we get more useful asserts of what is different
        check_len(entries_len);
    }
}
