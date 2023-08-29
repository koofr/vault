use std::sync::Arc;

use crate::{
    eventstream::EventStreamService,
    notifications::NotificationsService,
    oauth2::{errors::OAuth2Error, state::FinishFlowResult, OAuth2Service},
    remote::{Remote, RemoteError},
    repos::ReposService,
    secure_storage::SecureStorageService,
    space_usage::SpaceUsageService,
    store,
    user::UserService, common::state::Status,
};

use super::errors::LoadError;

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
                if let Err(err) = lifecycle_service.logout() {
                    notifications_service.show(format!("logout error: {:?}", err));
                }
            }
        }));

        lifecycle_service
    }

    pub async fn load(&self) -> Result<(), LoadError> {
        self.oauth2_service.load()?;

        if self.oauth2_service.is_authenticated() {
            self.on_login().await?;
        }

        Ok(())
    }

    pub async fn on_login(&self) -> Result<(), RemoteError> {
        self.eventstream_service.clone().connect();

        self.user_service.load_user().await?;
        self.repos_service.load_repos().await?;
        self.space_usage_service.load().await?;

        Ok(())
    }

    pub fn logout(&self) -> Result<(), OAuth2Error> {
        self.oauth2_service.logout()?;

        self.on_logout()
    }

    pub fn on_logout(&self) -> Result<(), OAuth2Error> {
        self.eventstream_service.disconnect();

        self.store.mutate(|state, notify, _, _| {
            state.reset();

            state.oauth2.status = Status::Initial;

            for event in store::Event::all() {
                notify(event);
            }
        });

        self.repos_service.reset();

        self.secure_storage_service.clear()?;

        Ok(())
    }

    pub async fn oauth2_finish_flow_url(&self, url: &str) -> Result<(), OAuth2Error> {
        match self.oauth2_service.finish_flow_url(url).await? {
            FinishFlowResult::LoggedIn => {
                self.on_login().await.map_err(|e| match e {
                    RemoteError::HttpError(err) => OAuth2Error::HttpError(err),
                    _ => OAuth2Error::Unknown(e.to_string()),
                })?;
            }
            FinishFlowResult::LoggedOut => {
                self.on_logout()?;
            }
        }

        Ok(())
    }
}
