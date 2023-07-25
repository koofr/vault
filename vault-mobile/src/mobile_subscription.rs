use std::{
    collections::{hash_map, HashMap},
    sync::{Arc, Mutex},
};

use vault_core::{store::Subscription, Vault};

use crate::SubscriptionCallback;

pub struct MobileSubscription {
    vault: Arc<Vault>,
    subscription: Subscription,
}

impl MobileSubscription {
    pub fn new(vault: Arc<vault_core::Vault>) -> Self {
        let subscription = vault.get_subscription();

        Self {
            vault,
            subscription,
        }
    }

    pub fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        callback: Box<dyn SubscriptionCallback>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>) -> T + Send + Sync + 'static,
    ) -> u32 {
        let callback = Box::new(move || callback.on_change());
        let vault = self.vault.clone();
        let generate_data = Box::new(move || generate_data(vault.clone()));

        self.subscription
            .subscribe(events, callback, subscription_data, generate_data)
    }

    pub fn subscribe_changed<T: Clone + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        callback: Box<dyn SubscriptionCallback>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>, hash_map::Entry<'_, u32, T>) -> bool
            + Send
            + Sync
            + 'static,
    ) -> u32 {
        let callback = Box::new(move || callback.on_change());
        let vault = self.vault.clone();
        let generate_data: Box<
            dyn Fn(hash_map::Entry<'_, u32, T>) -> bool + Send + Sync + 'static,
        > = Box::new(move |entry| generate_data(vault.clone(), entry));

        self.subscription
            .subscribe_changed(events, callback, subscription_data, generate_data)
    }

    pub fn get_data<T: Clone + Send>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Option<T> {
        self.subscription.get_data(id, subscription_data)
    }

    pub fn unsubscribe(&self, id: u32) {
        self.subscription.unsubscribe(id)
    }
}
