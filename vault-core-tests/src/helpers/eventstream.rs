use std::sync::Arc;

use vault_core::{
    eventstream::{self, state::MountSubscription},
    remote_files, store,
    types::{MountId, RemotePath},
};

pub struct EventstreamSubscription {
    pub mount_subscription: MountSubscription,
    pub store: Arc<store::Store>,
}

impl Drop for EventstreamSubscription {
    fn drop(&mut self) {
        self.store.mutate(|state, notify, mutation_state, _| {
            eventstream::mutations::remove_mount_subscriber(
                state,
                notify,
                mutation_state,
                self.mount_subscription.clone(),
            )
        });
    }
}

pub async fn eventstream_subscribe(
    store: Arc<store::Store>,
    mount_id: MountId,
    path: RemotePath,
    subscriber: &str,
) -> EventstreamSubscription {
    let mount_subscription = store.mutate(|state, notify, mutation_state, _| {
        eventstream::mutations::add_mount_subscriber(
            state,
            notify,
            mutation_state,
            mount_id.clone(),
            path.clone(),
            subscriber.to_owned(),
        )
    });

    eventstream_wait_registered(store.clone(), &mount_id, &path).await;

    EventstreamSubscription {
        mount_subscription,
        store,
    }
}

pub async fn eventstream_wait_registered(
    store: Arc<store::Store>,
    mount_id: &MountId,
    path: &RemotePath,
) {
    let file_id = remote_files::selectors::get_file_id(&mount_id, &path.to_lowercase());

    store::wait_for(store.clone(), &[store::Event::Eventstream], move |_| {
        if store.with_state(|state| {
            eventstream::selectors::select_mount_listener_registered_by_file_id(state, &file_id)
        }) {
            Some(())
        } else {
            None
        }
    })
    .await;
}
