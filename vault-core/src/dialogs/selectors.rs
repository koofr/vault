use crate::store;

use super::state::{Dialog, DialogInfo};

pub fn select_dialogs<'a>(state: &'a store::State) -> Vec<&'a Dialog> {
    let mut dialogs: Vec<&'a Dialog> = state.dialogs.dialogs.values().collect();

    dialogs.sort_by_key(|dialog| dialog.id);

    dialogs
}

pub fn select_dialog<'a>(state: &'a store::State, dialog_id: u32) -> Option<&'a Dialog> {
    state.dialogs.dialogs.get(&dialog_id)
}

pub fn select_dialog_mut<'a>(
    state: &'a mut store::State,
    dialog_id: u32,
) -> Option<&'a mut Dialog> {
    state.dialogs.dialogs.get_mut(&dialog_id)
}

pub fn select_dialog_info<'a>(state: &'a store::State, dialog_id: u32) -> Option<DialogInfo<'a>> {
    select_dialog(state, dialog_id).map(|dialog| DialogInfo {
        id: dialog.id,
        typ: &dialog.typ,
        title: &dialog.title,
        message: dialog.message.as_ref(),
        input_value: &dialog.input_value,
        is_input_value_valid: dialog.is_input_value_valid,
        input_value_selected: dialog.input_value_selected.as_ref(),
        input_placeholder: dialog.input_placeholder.as_ref(),
        confirm_button_text: &dialog.confirm_button_text,
        confirm_button_enabled: dialog.is_input_value_valid,
        confirm_button_style: &dialog.confirm_button_style,
        cancel_button_text: dialog.cancel_button_text.as_ref(),
    })
}
