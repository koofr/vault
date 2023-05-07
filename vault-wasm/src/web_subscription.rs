use std::{
    collections::{hash_map, HashMap},
    sync::{Arc, Mutex},
};

use vault_core::{store::Subscription, Vault};

pub struct WebSubscription {
    vault: Arc<Vault>,
    subscription: Subscription,
    window: web_sys::Window,
}

impl WebSubscription {
    pub fn new(vault: Arc<vault_core::Vault>) -> Self {
        let subscription = vault.get_subscription();

        Self {
            vault,
            subscription,
            window: web_sys::window().unwrap(),
        }
    }

    fn get_deferred_callback(
        &self,
        js_callback: js_sys::Function,
    ) -> Box<dyn Fn() + Send + Sync + 'static> {
        let window = self.window.clone();

        let callback: Box<dyn Fn() + 'static> = Box::new(move || {
            window.set_timeout_with_callback(&js_callback).unwrap();
        });

        let callback: Box<dyn Fn() + Send + Sync + 'static> = unsafe {
            Box::from_raw(Box::into_raw(callback) as *mut (dyn Fn() + Send + Sync + 'static))
        };

        callback
    }

    pub fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        js_callback: js_sys::Function,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>) -> T + 'static,
    ) -> u32 {
        let callback = self.get_deferred_callback(js_callback);
        let vault = self.vault.clone();
        let generate_data: Box<dyn Fn() -> T + 'static> =
            Box::new(move || generate_data(vault.clone()));
        let generate_data: Box<dyn Fn() -> T + Send + Sync> = unsafe {
            Box::from_raw(
                Box::into_raw(generate_data) as *mut (dyn Fn() -> T + Send + Sync + 'static)
            )
        };

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
        let vault = self.vault.clone();
        let generate_data: Box<dyn Fn(hash_map::Entry<'_, u32, T>) -> bool + 'static> =
            Box::new(move |entry| generate_data(vault.clone(), entry));
        let generate_data: Box<
            dyn Fn(hash_map::Entry<'_, u32, T>) -> bool + Send + Sync + 'static,
        > = unsafe {
            Box::from_raw(Box::into_raw(generate_data)
                as *mut (dyn Fn(hash_map::Entry<'_, u32, T>) -> bool
                     + Send
                     + Sync
                     + 'static))
        };

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
