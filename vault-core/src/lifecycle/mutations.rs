use crate::{common::state::Status, eventstream, store};

pub fn on_logout(state: &mut store::State, notify: &store::Notify) {
    state.reset();

    state.oauth2.status = Status::Initial;

    // state.reset() sets connection_state to Initial
    state.eventstream.connection_state = eventstream::state::ConnectionState::Disconnected;

    for event in store::Event::all() {
        notify(event);
    }
}
