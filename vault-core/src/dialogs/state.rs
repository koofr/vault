use std::collections::HashMap;

use crate::store::NextId;

#[derive(Debug, Clone)]
pub enum DialogType {
    Alert,
    Confirm,
    Prompt,
}

#[derive(Debug, Clone)]
pub enum DialogButtonStyle {
    Primary,
    Destructive,
}

pub struct DialogShowOptions {
    pub typ: DialogType,
    pub title: String,
    pub message: Option<String>,
    pub input_value: String,
    pub input_value_validator: Option<Box<dyn Fn(&str) -> bool + Send + Sync + 'static>>,
    pub input_value_selected: Option<String>,
    pub input_placeholder: Option<String>,
    pub confirm_button_text: String,
    pub confirm_button_style: DialogButtonStyle,
    pub cancel_button_text: Option<String>,
}

pub struct DialogInfo<'a> {
    pub id: u32,
    pub typ: &'a DialogType,
    pub title: &'a String,
    pub message: Option<&'a String>,
    pub input_value: &'a String,
    pub is_input_value_valid: bool,
    pub input_value_selected: Option<&'a String>,
    pub input_placeholder: Option<&'a String>,
    pub confirm_button_text: &'a String,
    pub confirm_button_enabled: bool,
    pub confirm_button_style: &'a DialogButtonStyle,
    pub cancel_button_text: Option<&'a String>,
}

#[derive(Debug, Clone)]
pub struct Dialog {
    pub id: u32,
    pub typ: DialogType,
    pub title: String,
    pub message: Option<String>,
    pub input_value: String,
    pub is_input_value_valid: bool,
    pub input_value_selected: Option<String>,
    pub input_placeholder: Option<String>,
    pub confirm_button_text: String,
    pub confirm_button_style: DialogButtonStyle,
    pub cancel_button_text: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DialogsState {
    pub dialogs: HashMap<u32, Dialog>,
    pub next_id: NextId,
}

impl DialogsState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
