use std::{
    collections::{hash_map, HashMap},
    sync::{Arc, Mutex},
};

use vault_core::subscription::Subscription;

pub struct WebSubscription {
    subscription: Subscription,
    window: web_sys::Window,
}

impl WebSubscription {
    pub fn new(vault: Arc<vault_core::Vault>) -> Self {
        Self {
            subscription: Subscription::new(vault),
            window: web_sys::window().unwrap(),
        }
    }

    fn get_deferred_callback(&self, js_callback: js_sys::Function) -> Box<dyn Fn() + 'static> {
        let window = self.window.clone();

        Box::new(move || {
            window.set_timeout_with_callback(&js_callback).unwrap();
        })
    }

    pub fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        js_callback: js_sys::Function,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>) -> T + 'static,
    ) -> u32 {
        let callback = self.get_deferred_callback(js_callback);

        self.subscription
            .subscribe(events, callback, subscription_data, generate_data)
    }

    pub fn subscribe_changed<T: Clone + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        js_callback: js_sys::Function,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>, hash_map::Entry<'_, u32, T>) -> bool + 'static,
    ) -> u32 {
        let callback = self.get_deferred_callback(js_callback);

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
