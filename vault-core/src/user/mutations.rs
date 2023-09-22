use crate::{common::state::Status, remote, store};

use super::state::User;

pub fn loading(state: &mut store::State, notify: &store::Notify) {
    notify(store::Event::User);

    state.user.status = Status::Loading {
        loaded: state.user.status.loaded(),
    };
}

pub fn loaded(
    state: &mut store::State,
    notify: &store::Notify,
    res: Result<remote::models::User, remote::RemoteError>,
) {
    notify(store::Event::User);

    match res {
        Ok(user) => {
            let full_name = match (user.first_name.as_str(), user.last_name.as_str()) {
                ("", "") => user.email.clone(),
                (first_name, "") => first_name.to_owned(),
                ("", last_name) => last_name.to_owned(),
                (first_name, last_name) => format!("{} {}", first_name, last_name),
            };

            state.user.user = Some(User {
                id: user.id,
                first_name: user.first_name,
                last_name: user.last_name,
                full_name,
                email: user.email,
                profile_picture_status: state
                    .user
                    .user
                    .as_ref()
                    .map(|x| x.profile_picture_status.clone())
                    .unwrap_or(Status::Initial),
                profile_picture_bytes: state
                    .user
                    .user
                    .as_ref()
                    .and_then(|x| x.profile_picture_bytes.clone()),
            });

            state.user.status = Status::Loaded;
        }
        Err(err) => {
            state.user.status = Status::Error {
                error: err.clone(),
                loaded: state.user.status.loaded(),
            };
        }
    }
}
