use std::{
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};

use futures::channel::oneshot;

use super::Store;

/// Waits until `f` returns Some. If you call `store.mutate` in `f`, `notify`
/// must not be called if `mutate` returns `None` otherwise `f` will be called
/// again and cause an infinite recursion.
///
/// ```ignore
/// wait_for(store, &[Event::MyEvent], move |unsubscribe| {
///     store.mutate(|state, notify, _, _| {
///         if is_already_saving(state) {
///             return None;
///         }
///
///         save(state);
///
///         notify(Event::MyEvent);
///
///         Some(())
///     });
/// });
/// ```
pub async fn wait_for<F, R, State, Event, MutationState, MutationEvent>(
    store: Arc<Store<State, Event, MutationState, MutationEvent>>,
    events: &[Event],
    f: F,
) -> R
where
    F: Fn() -> Option<R> + Send + Sync + 'static,
    R: Send + 'static,
    State: Debug + Clone + Send + Sync + 'static,
    Event: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
    MutationState: Debug + Clone + Default + 'static,
    MutationEvent: Debug + Clone + PartialEq + Eq + Hash + Send + 'static,
{
    if let Some(res) = f() {
        return res;
    }

    let subscription_id = store.get_next_id();

    let (sender, receiver) = oneshot::channel();

    let subscription_sender = Arc::new(Mutex::new(Some(sender)));

    let subscription_store = store.clone();

    let f = Arc::new(Box::new(f));

    let subscription_f = f.clone();

    store.on(
        subscription_id,
        events,
        Box::new(move |_, _| {
            let subscription_f = subscription_f.clone();

            if let Some(res) = subscription_f() {
                let sender = subscription_sender.lock().unwrap().take();

                subscription_store.remove_listener(subscription_id);

                if let Some(sender) = sender {
                    let _ = sender.send(res);
                }
            }
        }),
    );

    // try again in case state changed between the first check and subscribe
    if let Some(res) = f() {
        store.remove_listener(subscription_id);

        return res;
    }

    receiver.await.unwrap()
}
