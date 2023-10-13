use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::channel::oneshot;

use crate::store;

use super::{
    mutations, selectors,
    state::{DialogButtonStyle, DialogShowOptions, DialogType},
};

pub struct DialogsService {
    store: Arc<store::Store>,
    input_value_validators:
        Arc<RwLock<HashMap<u32, Box<dyn Fn(String) -> bool + Send + Sync + 'static>>>>,
    results: Arc<RwLock<HashMap<u32, oneshot::Sender<Option<String>>>>>,
}

impl DialogsService {
    pub fn new(store: Arc<store::Store>) -> Self {
        Self {
            store,
            input_value_validators: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn build_alert(&self, title: String) -> DialogShowOptions {
        DialogShowOptions {
            typ: DialogType::Alert,
            title,
            message: None,
            input_value: String::from(""),
            input_value_selected: None,
            input_placeholder: None,
            confirm_button_text: String::from("Ok"),
            confirm_button_style: DialogButtonStyle::Primary,
            cancel_button_text: None,
        }
    }

    pub fn build_confirm(&self) -> DialogShowOptions {
        DialogShowOptions {
            typ: DialogType::Confirm,
            title: String::from("Are you sure?"),
            message: None,
            input_value: String::from(""),
            input_value_selected: None,
            input_placeholder: None,
            confirm_button_text: String::from("Yes"),
            confirm_button_style: DialogButtonStyle::Primary,
            cancel_button_text: Some(String::from("No")),
        }
    }

    pub fn build_prompt(&self, title: String) -> DialogShowOptions {
        DialogShowOptions {
            typ: DialogType::Prompt,
            title,
            message: None,
            input_value: String::from(""),
            input_value_selected: None,
            input_placeholder: None,
            confirm_button_text: String::from("Ok"),
            confirm_button_style: DialogButtonStyle::Primary,
            cancel_button_text: Some(String::from("Cancel")),
        }
    }

    pub async fn show(&self, options: DialogShowOptions) -> Option<String> {
        let dialog_id = self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Dialogs);

            mutations::get_next_id(state)
        });

        let (result_sender, result_receiver) = oneshot::channel();

        self.results
            .write()
            .unwrap()
            .insert(dialog_id, result_sender);

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Dialogs);

            mutations::show(state, dialog_id, options, true)
        });

        result_receiver.await.unwrap()
    }

    pub async fn show_validator<Error, Validator>(
        &self,
        options: DialogShowOptions,
        input_value_validator: Validator,
    ) -> Option<Result<String, Error>>
    where
        Error: std::error::Error,
        Validator: Fn(String) -> Result<String, Error> + Send + Sync + 'static,
    {
        let dialog_id = self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Dialogs);

            mutations::get_next_id(state)
        });

        let input_value_validator = Arc::new(input_value_validator);

        let is_input_value_valid = {
            let validator = input_value_validator.clone();
            let validator = Box::new(move |value| (validator.as_ref())(value).is_ok());

            let is_input_value_valid = validator(options.input_value.clone());

            self.input_value_validators
                .write()
                .unwrap()
                .insert(dialog_id, validator);

            is_input_value_valid
        };

        let (result_sender, result_receiver) = oneshot::channel();

        self.results
            .write()
            .unwrap()
            .insert(dialog_id, result_sender);

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Dialogs);

            mutations::show(state, dialog_id, options, is_input_value_valid)
        });

        result_receiver
            .await
            .unwrap()
            .map(|value| (input_value_validator.as_ref())(value))
    }

    pub fn remove(&self, dialog_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Dialogs);

            mutations::remove(state, dialog_id);
        });

        self.input_value_validators
            .write()
            .unwrap()
            .remove(&dialog_id);
    }

    pub fn confirm(&self, dialog_id: u32) {
        let value = match self.store.with_state(|state| {
            selectors::select_dialog(state, dialog_id).map(|dialog| dialog.input_value.clone())
        }) {
            Some(x) => x,
            None => return,
        };

        if let Some(sender) = self.results.write().unwrap().remove(&dialog_id) {
            let _ = sender.send(Some(value));
        }

        self.remove(dialog_id);
    }

    pub fn cancel(&self, dialog_id: u32) {
        if let Some(sender) = self.results.write().unwrap().remove(&dialog_id) {
            let _ = sender.send(None);
        }

        self.remove(dialog_id);
    }

    pub fn set_input_value(&self, dialog_id: u32, value: String) {
        let is_valid = {
            let input_value_validators = self.input_value_validators.read().unwrap();
            let input_value_validator = input_value_validators.get(&dialog_id);

            input_value_validator
                .map(|validator| validator(value.clone()))
                .unwrap_or(true)
        };

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::Dialogs);

            mutations::set_input_value(state, dialog_id, value, is_valid);
        });
    }
}
