use crate::{lifecycle::state::AppVisibility, store};

pub fn select_is_visible(state: &store::State) -> bool {
    matches!(state.lifecycle.app_visibility, AppVisibility::Visible)
}
