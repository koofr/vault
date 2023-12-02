use std::sync::Arc;

use futures::{
    future::{self, BoxFuture},
    join, FutureExt, TryFutureExt,
};

use crate::{
    eventstream::EventStreamService,
    notifications::NotificationsService,
    oauth2::{state::FinishFlowResult, OAuth2Service},
    remote::Remote,
    repos::ReposService,
    secure_storage::SecureStorageService,
    space_usage::SpaceUsageService,
    store,
    user::UserService,
};

use super::{
    errors::{LoadError, LogoutError, OAuth2FinishFlowUrlError, OnLoginError, OnLogoutError},
    mutations,
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

    /// BoxFuture is used so that calling load immediately loads oauth2 service
    /// and then loads the rest asynchronously
    pub fn load(self: Arc<Self>) -> Result<BoxFuture<'static, Result<(), LoadError>>, LoadError> {
        self.oauth2_service
            .load()
            .map_err(LoadError::OAuth2LoadError)?;

        let load_future: BoxFuture<'static, Result<(), LoadError>> =
            if self.oauth2_service.is_authenticated() {
                let on_login_self = self.clone();

                async move {
                    on_login_self
                        .on_login()
                        .map_err(LoadError::OnLoginError)
                        .await
                }
                .boxed()
            } else {
                future::ready(Ok(())).boxed()
            };

        Ok(load_future)
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
            mutations::on_logout(state, notify);
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

    pub fn app_visible(&self) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::app_visible(state, notify, mutation_state, mutation_notify);
            })
    }

    pub fn app_hidden(&self) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::app_hidden(state, notify, mutation_state, mutation_notify);
            })
    }
}

impl Drop for LifecycleService {
    fn drop(&mut self) {
        self.eventstream_service.disconnect();
    }
}
