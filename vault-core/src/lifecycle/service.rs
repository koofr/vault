use std::sync::Arc;

use futures::{join, TryFutureExt};

use crate::{
    common::state::Status,
    eventstream::{self, EventStreamService},
    notifications::NotificationsService,
    oauth2::{state::FinishFlowResult, OAuth2Service},
    remote::Remote,
    repos::ReposService,
    secure_storage::SecureStorageService,
    space_usage::SpaceUsageService,
    store,
    user::UserService,
};

use super::errors::{
    LoadError, LogoutError, OAuth2FinishFlowUrlError, OnLoginError, OnLogoutError,
};

pub struct LifecycleService {
    secure_storage_service: Arc<SecureStorageService>,
    oauth2_service: Arc<OAuth2Service>,
    user_service: Arc<UserService>,
    repos_service: Arc<ReposService>,
    eventstream_service: Arc<EventStreamService>,
    space_usage_service: Arc<SpaceUsageService>,
    store: Arc<store::Store>,
}

impl LifecycleService {
    pub fn new(
        secure_storage_service: Arc<SecureStorageService>,
        notifications_service: Arc<NotificationsService>,
        oauth2_service: Arc<OAuth2Service>,
        user_service: Arc<UserService>,
        repos_service: Arc<ReposService>,
        eventstream_service: Arc<EventStreamService>,
        space_usage_service: Arc<SpaceUsageService>,
        remote: Arc<Remote>,
        store: Arc<store::Store>,
    ) -> Arc<Self> {
        let lifecycle_service = Arc::new(Self {
            secure_storage_service,
            oauth2_service,
            user_service,
            repos_service,
            eventstream_service,
            space_usage_service,
            store,
        });

        let remote_logout_lifecycle_service = Arc::downgrade(&lifecycle_service);

        remote.set_logout(Box::new(move || {
            if let Some(lifecycle_service) = remote_logout_lifecycle_service.upgrade() {
                match lifecycle_service.logout() {
                    Ok(()) => notifications_service
                        .show("You've been logged out. Please log in again.".into()),
                    Err(err) => notifications_service.show(format!("logout error: {:?}", err)),
                }
            }
        }));

        lifecycle_service
    }

    pub async fn load(&self) -> Result<(), LoadError> {
        self.oauth2_service
            .load()
            .map_err(LoadError::OAuth2LoadError)?;

        if self.oauth2_service.is_authenticated() {
            self.on_login().await.map_err(LoadError::OnLoginError)?;
        }

        Ok(())
    }

    pub async fn on_login(&self) -> Result<(), OnLoginError> {
        self.eventstream_service.clone().connect();

        let user_future = self
            .user_service
            .load_user()
            .map_err(OnLoginError::LoadUserError);
        let repos_future = self
            .repos_service
            .load_repos()
            .map_err(OnLoginError::LoadReposError);
        let space_usage_future = self
            .space_usage_service
            .load()
            .map_err(OnLoginError::LoadSpaceUsageError);

        let (user_res, repos_res, space_usage_res) =
            join!(user_future, repos_future, space_usage_future);

        user_res?;
        repos_res?;
        space_usage_res?;

        Ok(())
    }

    pub fn logout(&self) -> Result<(), LogoutError> {
        self.oauth2_service
            .logout()
            .map_err(LogoutError::OAuth2LogoutError)?;

        self.on_logout()?;

        Ok(())
    }

    pub fn on_logout(&self) -> Result<(), OnLogoutError> {
        self.eventstream_service.disconnect();

        self.store.mutate(|state, notify, _, _| {
            state.reset();

            state.oauth2.status = Status::Initial;

            // state.reset() sets connection_state to Initial
            state.eventstream.connection_state = eventstream::state::ConnectionState::Disconnected;

            for event in store::Event::all() {
                notify(event);
            }
        });

        self.secure_storage_service
            .clear()
            .map_err(OnLogoutError::ClearStorageError)?;

        Ok(())
    }

    pub async fn oauth2_finish_flow_url(&self, url: &str) -> Result<(), OAuth2FinishFlowUrlError> {
        match self.oauth2_service.finish_flow_url(url).await? {
            FinishFlowResult::LoggedIn => {
                self.on_login().await?;
            }
            FinishFlowResult::LoggedOut => {
                self.on_logout()?;
            }
        }

        Ok(())
    }
}

impl Drop for LifecycleService {
    fn drop(&mut self) {
        self.eventstream_service.disconnect();
    }
}
