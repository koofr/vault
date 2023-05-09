use std::sync::{Arc, Mutex};

use futures::channel::oneshot;

use super::{Event, Store};

/// Waits until `f` returns Some. If you need to call `store.mutate` inside `f`,
/// you need to use `store.mutate_notify()` and only call `notify` when `f`
/// returns `Some` otherwise `mutate` will emit an event which will call the `f`
/// again and cause an infinite recursion.
///
/// ```ignore
/// wait_for(store, &[Event::MyEvent], move |unsubscribe| {
///     store.mutate_notify(|state, notify| {
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
pub async fn wait_for<F: Fn() -> Option<R> + Send + Sync + 'static, R: Send + 'static>(
    store: Arc<Store>,
    events: &[Event],
    f: F,
) -> R {
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
        Box::new(move || {
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
