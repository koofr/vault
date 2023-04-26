use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{store, Vault};

pub struct Subscription {
    vault: Arc<Vault>,
    cleanups: Arc<Mutex<HashMap<u32, Box<dyn Fn() + Send + Sync + 'static>>>>,
}

impl Subscription {
    pub fn new(vault: Arc<Vault>) -> Self {
        Self {
            vault,
            cleanups: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[store::Event],
        callback: Box<dyn Fn() + 'static>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<Vault>) -> T + 'static,
    ) -> u32 {
        let id = self.vault.get_next_id();

        let generate_data = Arc::new(generate_data);

        let callback_vault = self.vault.clone();
        let callback_subscription_data = subscription_data.clone();
        let callback_generate_data = generate_data.clone();

        let store_callback: Box<dyn Fn() + 'static> = Box::new(move || {
            let new_value = callback_generate_data(callback_vault.clone());

            let callback_subscription_data = callback_subscription_data.clone();
            let mut subscription_data = callback_subscription_data.lock().unwrap();
            let current_data = subscription_data.get(&id);

            if current_data.is_none() || new_value != *current_data.unwrap() {
                subscription_data.insert(id, new_value.clone());

                // unlock subscription_data before calling the callback
                drop(subscription_data);

                callback();
            }
        });
        let store_callback: Box<dyn Fn() + Send + Sync + 'static> = unsafe {
            Box::from_raw(Box::into_raw(store_callback) as *mut (dyn Fn() + Send + Sync + 'static))
        };

        self.vault.on(id, events, store_callback);

        subscription_data
            .lock()
            .unwrap()
            .insert(id, generate_data(self.vault.clone()));

        let cleanup_subscription_data = subscription_data.clone();

        let cleanup = Box::new(move || {
            cleanup_subscription_data
                .clone()
                .lock()
                .unwrap()
                .remove(&id);
        });

        self.cleanups.lock().unwrap().insert(id, cleanup);

        id
    }

    pub fn get_data<T: Clone + PartialEq + Send>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Option<T> {
        subscription_data.lock().unwrap().get(&id).cloned()
    }

    pub fn unsubscribe(&self, id: u32) {
        self.vault.remove_listener(id);

        let cleanup = self.cleanups.lock().unwrap().remove(&id);

        if let Some(cleanup) = cleanup {
            cleanup();
        }
    }
}
