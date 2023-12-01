use crate::{common::state::Status, eventstream, store};

use super::state::AppVisibility;

pub fn on_logout(state: &mut store::State, notify: &store::Notify) {
    state.reset();

    state.oauth2.status = Status::Initial;

    // state.reset() sets connection_state to Initial
    state.eventstream.connection_state = eventstream::state::ConnectionState::Disconnected;

    for event in store::Event::all() {
        notify(event);
    }
}

pub fn app_visible(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
) {
    if matches!(state.lifecycle.app_visibility, AppVisibility::Visible) {
        return;
    }

    state.lifecycle.app_visibility = AppVisibility::Visible;

    notify(store::Event::Lifecycle);

    mutation_notify(store::MutationEvent::Lifecycle, state, mutation_state);
}

pub fn app_hidden(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
) {
    if matches!(state.lifecycle.app_visibility, AppVisibility::Hidden) {
        return;
    }

    state.lifecycle.app_visibility = AppVisibility::Hidden;

    notify(store::Event::Lifecycle);

    mutation_notify(store::MutationEvent::Lifecycle, state, mutation_state);
}
