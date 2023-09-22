use std::sync::Arc;

use crate::{common::state::Status, remote, store};

use super::mutations;

pub struct UserService {
    remote: Arc<remote::Remote>,
    store: Arc<store::Store>,
}

impl UserService {
    pub fn new(remote: Arc<remote::Remote>, store: Arc<store::Store>) -> Self {
        Self { remote, store }
    }

    pub async fn load_user(&self) -> Result<(), remote::RemoteError> {
        self.store.mutate(|state, notify, _, _| {
            mutations::loading(state, notify);
        });

        let res = self.remote.get_user().await;

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store
            .mutate(|state, notify, _, _| mutations::loaded(state, notify, res));

        res_err
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

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::User);

            if let Some(ref mut user) = state.user.user {
                user.profile_picture_status = Status::Loading {
                    loaded: user.profile_picture_status.loaded(),
                };
            }
        });

        let profile_picture_bytes = match self.remote.get_profile_picture_bytes(&user_id).await {
            Ok(bytes) => Some(bytes),
            Err(remote::RemoteError::ApiError {
                code: remote::ApiErrorCode::NotFound,
                ..
            }) => None,
            Err(err) => {
                self.store.mutate(|state, notify, _, _| {
                    notify(store::Event::User);

                    if let Some(ref mut user) = state.user.user {
                        user.profile_picture_status = Status::Error {
                            error: err.clone(),
                            loaded: user.profile_picture_status.loaded(),
                        };
                    }
                });

                return Err(err);
            }
        };

        self.store.mutate(|state, notify, _, _| {
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
