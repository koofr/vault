use crate::{common::state::Status, store};

use super::{
    selectors,
    state::{Dialog, DialogShowOptions},
};

pub fn get_next_id(state: &mut store::State) -> u32 {
    let dialog_id = state.dialogs.next_id;

    state.dialogs.next_id += 1;

    dialog_id
}

pub fn show(
    state: &mut store::State,
    dialog_id: u32,
    options: DialogShowOptions,
    is_input_value_valid: bool,
) {
    let dialog = Dialog {
        id: dialog_id,
        status: Status::Initial,
        typ: options.typ,
        title: options.title,
        message: options.message,
        input_value: options.input_value,
        is_input_value_valid,
        input_value_selected: options.input_value_selected,
        input_placeholder: options.input_placeholder,
        confirm_button_text: options.confirm_button_text,
        confirm_button_style: options.confirm_button_style,
        cancel_button_text: options.cancel_button_text,
    };

    state.dialogs.dialogs.insert(dialog_id, dialog);
}

pub fn remove(state: &mut store::State, dialog_id: u32) {
    state.dialogs.dialogs.remove(&dialog_id);
}

pub fn set_input_value(state: &mut store::State, dialog_id: u32, value: String, is_valid: bool) {
    let dialog = match selectors::select_dialog_mut(state, dialog_id) {
        Some(dialog) => dialog,
        None => return,
    };

    dialog.input_value = value;
    dialog.is_input_value_valid = is_valid;
    dialog.input_value_selected = None;
}
