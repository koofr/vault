use crate::{common::state::Status, store};

use super::{errors::OAuth2Error, state::OAuth2Token};

pub fn loaded(state: &mut store::State, notify: &store::Notify, token: Option<OAuth2Token>) {
    notify(store::Event::Auth);

    state.oauth2.status = match token {
        Some(_) => Status::Loaded,
        None => Status::Initial,
    };

    state.oauth2.token = token;
}

pub fn logout(state: &mut store::State, notify: &store::Notify) {
    notify(store::Event::Auth);

    state.oauth2.status = Status::Initial;
    state.oauth2.token = None;
}

pub fn update_token(state: &mut store::State, notify: &store::Notify, token: OAuth2Token) {
    notify(store::Event::Auth);

    state.oauth2.token = Some(token.clone());
}

pub fn error(state: &mut store::State, notify: &store::Notify, err: OAuth2Error) {
    notify(store::Event::Auth);

    state.oauth2.status = Status::Error { error: err };
}

pub fn logging_in(state: &mut store::State, notify: &store::Notify) {
    notify(store::Event::Auth);

    state.oauth2.status = Status::Loading;
}

pub fn logged_in(state: &mut store::State, notify: &store::Notify, token: OAuth2Token) {
    notify(store::Event::Auth);

    state.oauth2.status = Status::Loaded;
    state.oauth2.token = Some(token);
}