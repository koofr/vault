use std::sync::Arc;

use crate::{common::state::Status, remote, store};

use super::state::User;

pub struct UserService {
    remote: Arc<remote::Remote>,
    store: Arc<store::Store>,
}

impl UserService {
    pub fn new(remote: Arc<remote::Remote>, store: Arc<store::Store>) -> Self {
        Self { remote, store }
    }

    pub async fn load_user(&self) -> Result<(), remote::RemoteError> {
        self.store.mutate(|state, notify| {
            notify(store::Event::User);

            state.user.status = Status::Loading;
        });

        let user = match self.remote.get_user().await {
            Ok(user) => user,
            Err(err) => {
                self.store.mutate(|state, notify| {
                    notify(store::Event::User);

                    state.user.status = Status::Error { error: err.clone() };
                });

                return Err(err);
            }
        };

        self.store.mutate(|state, notify| {
            notify(store::Event::User);

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
                profile_picture_status: Status::Initial,
                profile_picture_bytes: None,
            });

            state.user.status = Status::Loaded;
        });

        Ok(())
    }

    pub async fn load_profile_picture(&self) -> Result<(), remote::RemoteError> {
        let user_id = match self
            .store
            .with_state(|state| state.user.user.as_ref().map(|user| user.id.clone()))
        {
            Some(user_id) => user_id,
            None => {
                return Ok(());
            }
        };

        self.store.mutate(|state, notify| {
            notify(store::Event::User);

            if let Some(ref mut user) = state.user.user {
                user.profile_picture_status = Status::Loading;
            }
        });

        let profile_picture_bytes = match self.remote.get_profile_picture_bytes(&user_id).await {
            Ok(bytes) => Some(bytes),
            Err(remote::RemoteError::ApiError {
                code: remote::ApiErrorCode::NotFound,
                ..
            }) => None,
            Err(err) => {
                self.store.mutate(|state, notify| {
                    notify(store::Event::User);

                    if let Some(ref mut user) = state.user.user {
                        user.profile_picture_status = Status::Error { error: err.clone() };
                    }
                });

                return Err(err);
            }
        };

        self.store.mutate(|state, notify| {
            notify(store::Event::User);

            if let Some(ref mut user) = state.user.user {
                user.profile_picture_status = Status::Loaded;
                user.profile_picture_bytes = profile_picture_bytes;
            }
        });

        Ok(())
    }

    pub async fn ensure_profile_picture(&self) -> Result<(), remote::RemoteError> {
        if self.store.with_state(|state| {
            state
                .user
                .user
                .as_ref()
                .map(|user| match user.profile_picture_status {
                    Status::Initial => true,
                    _ => false,
                })
                .unwrap_or(false)
        }) {
            return self.load_profile_picture().await;
        }

        Ok(())
    }
}
