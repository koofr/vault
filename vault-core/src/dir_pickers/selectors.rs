use serde::de::DeserializeOwned;

use crate::store;

use super::state::{DirPicker, DirPickerFileId, DirPickerItem, DirPickerItemId};

pub fn select_picker<'a>(state: &'a store::State, picker_id: u32) -> Option<&'a DirPicker> {
    state.dir_pickers.pickers.get(&picker_id)
}

pub fn select_picker_mut<'a>(
    state: &'a mut store::State,
    picker_id: u32,
) -> Option<&'a mut DirPicker> {
    state.dir_pickers.pickers.get_mut(&picker_id)
}

pub fn select_picker_options<Options>(picker: &DirPicker) -> Options
where
    Options: DeserializeOwned,
{
    serde_json::from_value(picker.options.clone()).unwrap()
}

pub fn select_options<Options>(state: &store::State, picker_id: u32) -> Option<Options>
where
    Options: DeserializeOwned,
{
    select_picker(state, picker_id).map(|picker| select_picker_options(&picker))
}

pub fn select_item<'a>(
    state: &'a store::State,
    picker_id: u32,
    item_id: &DirPickerItemId,
) -> Option<&'a DirPickerItem> {
    select_picker(state, picker_id)
        .and_then(|picker| picker.items.iter().find(|item| &item.id == item_id))
}

pub fn select_is_open(state: &store::State, picker_id: u32, item_id: &DirPickerItemId) -> bool {
    select_picker(state, picker_id)
        .map(|picker| picker.open_ids.contains(item_id))
        .unwrap_or(false)
}

pub fn select_selected_id<'a>(
    state: &'a store::State,
    picker_id: u32,
) -> Option<&'a DirPickerItemId> {
    select_picker(state, picker_id).and_then(|picker| picker.selected_id.as_ref())
}

pub fn select_selected_file_id<'a>(
    state: &'a store::State,
    picker_id: u32,
) -> Option<&'a DirPickerFileId> {
    select_selected_id(state, picker_id)
        .and_then(|item_id| select_item(state, picker_id, item_id))
        .and_then(|item| item.file_id.as_ref())
}

pub fn select_is_selected<'a>(
    state: &'a store::State,
    picker_id: u32,
    item_id: &DirPickerItemId,
) -> bool {
    select_selected_id(state, picker_id) == Some(item_id)
}
