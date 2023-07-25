use std::{fmt::Debug, hash::Hash, sync::Arc};

use crate::Store;

pub struct RemoveListener(Option<Box<dyn Fn() + Send + Sync + 'static>>);

impl RemoveListener {
    pub fn new<State, Event, MutationState, MutationEvent>(
        store: Arc<Store<State, Event, MutationState, MutationEvent>>,
        id: u32,
    ) -> Self
    where
        State: Debug + Clone + Send + Sync + 'static,
        Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
        MutationState: Debug + Clone + Default + 'static,
        MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    {
        Self(Some(Box::new(move || {
            store.remove_listener(id);
        })))
    }
}

impl Drop for RemoveListener {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}
