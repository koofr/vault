use crate::{common::state::Status, store};

use super::errors::OAuth2Error;

pub fn select_status<'a>(state: &'a store::State) -> &'a Status<OAuth2Error> {
    &state.oauth2.status
}

pub fn select_is_authenticated(state: &store::State) -> bool {
    state.oauth2.token.is_some()
}
