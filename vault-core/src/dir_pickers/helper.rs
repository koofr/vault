use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{future::BoxFuture, FutureExt};
use serde::Serialize;

use crate::{dir_pickers::state::DirPicker, store};

use super::{selectors, state::DirPickerItem};

pub struct DirPickersHelper<E> {
    store: Arc<store::Store>,
    select_items: Box<dyn Fn(&store::State, &DirPicker) -> Vec<DirPickerItem> + Send + Sync>,
    on_expand: Box<dyn Fn(u32, DirPickerItem) -> BoxFuture<'static, Result<(), E>> + Send + Sync>,
    listeners: Arc<Mutex<HashMap<u32, u32>>>,
}

impl<E: Send + Sync + 'static> DirPickersHelper<E> {
    pub fn new(
        store: Arc<store::Store>,
        select_items: Box<dyn Fn(&store::State, &DirPicker) -> Vec<DirPickerItem> + Send + Sync>,
        on_expand: Box<
            dyn Fn(u32, DirPickerItem) -> BoxFuture<'static, Result<(), E>> + Send + Sync,
        >,
    ) -> Self {
        Self {
            store,
            select_items,
            on_expand,
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create<Options>(self: Arc<Self>, events: &[store::Event], options: Options) -> u32
    where
        Options: Serialize,
    {
        let options = serde_json::to_value(options).unwrap();

        let picker_id = self.store.mutate(|state, notify, _, _| {
            notify(store::Event::DirPickers);

            let picker_id = state.dir_pickers.next_id.next();

            let picker = DirPicker {
                id: picker_id,
                options,
                ..DirPicker::default()
            };

            state.dir_pickers.pickers.insert(picker_id, picker);

            picker_id
        });

        self.generate_items(picker_id);

        let update_items_self = self.clone();

        let listener_id = self.store.get_next_id();

        self.store.on(
            listener_id,
            events,
            Box::new(move |_, _| {
                update_items_self.generate_items(picker_id);
            }),
        );

        self.listeners
            .lock()
            .unwrap()
            .insert(picker_id, listener_id);

        picker_id
    }

    pub fn destroy(&self, picker_id: u32) {
        if let Some(listener_id) = self.listeners.lock().unwrap().remove(&picker_id) {
            self.store.remove_listener(listener_id);
        }

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::DirPickers);

            state.dir_pickers.pickers.remove(&picker_id);
        });
    }

    pub fn mutate_picker<F>(&self, picker_id: u32, f: F)
    where
        F: FnOnce(&mut DirPicker),
    {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::DirPickers);

            if let Some(picker) = selectors::select_picker_mut(state, picker_id) {
                f(picker)
            }
        });

        self.generate_items(picker_id);
    }

    pub async fn click(&self, picker_id: u32, item_id: &str, is_arrow: bool) -> Result<(), E> {
        let item = match self.get_item(picker_id, item_id) {
            Some(item) => item,
            None => return Ok(()),
        };

        if !is_arrow {
            self.item_set_selected(picker_id, &item);
        }

        if item.is_open {
            if is_arrow || item.is_selected || item.file_id.is_none() {
                self.item_collapse(picker_id, &item);
            }
        } else {
            self.item_expand(picker_id, item, true).await?;
        }

        Ok(())
    }

    pub fn set_selected(&self, picker_id: u32, item_id: &str) {
        let item = match self.get_item(picker_id, item_id) {
            Some(item) => item,
            None => return,
        };

        self.item_set_selected(picker_id, &item)
    }

    fn item_set_selected(&self, picker_id: u32, item: &DirPickerItem) {
        if !item.is_selectable || item.is_selected {
            return;
        }

        self.mutate_picker(picker_id, |picker| {
            picker.selected_id = Some(item.id.clone());
        });
    }

    pub fn expand(&self, picker_id: u32, item_id: &str, force: bool) -> BoxFuture<Result<(), E>> {
        let item = match self.get_item(picker_id, item_id) {
            Some(item) => item,
            None => return futures::future::ready(Ok(())).boxed(),
        };

        self.item_expand(picker_id, item, force)
    }

    fn item_expand(
        &self,
        picker_id: u32,
        item: DirPickerItem,
        force: bool,
    ) -> BoxFuture<Result<(), E>> {
        if item.is_open && !force {
            return futures::future::ready(Ok(())).boxed();
        }

        let item_id = item.id.clone();

        self.mutate_picker(picker_id, |picker| {
            picker.open_ids.insert(item_id.clone());
        });

        (async move {
            self.set_loading(picker_id, &item_id, true);

            let res = (self.on_expand)(picker_id, item).await;

            self.set_loading(picker_id, &item_id, false);

            res
        })
        .boxed()
    }

    fn set_loading(&self, picker_id: u32, item_id: &str, is_loading: bool) {
        self.mutate_picker(picker_id, |picker| {
            if is_loading {
                picker.loading_ids.insert(item_id.to_owned());
            } else {
                picker.loading_ids.remove(item_id);
            }
        });
    }

    fn item_collapse(&self, picker_id: u32, item: &DirPickerItem) {
        self.mutate_picker(picker_id, |picker| {
            picker.open_ids.remove(&item.id);
        });
    }

    fn generate_items(&self, picker_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::DirPickers);

            if let Some(items) = selectors::select_picker(state, picker_id)
                .map(|picker| (self.select_items)(state, picker))
            {
                if let Some(picker) = selectors::select_picker_mut(state, picker_id) {
                    picker.items = items;
                }
            }
        });
    }

    fn get_item(&self, picker_id: u32, item_id: &str) -> Option<DirPickerItem> {
        self.store
            .with_state(|state| selectors::select_item(state, picker_id, item_id).cloned())
    }
}
