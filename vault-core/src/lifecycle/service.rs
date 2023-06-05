use std::sync::Arc;

use crate::{
    eventstream::EventStreamService,
    oauth2::OAuth2Service,
    remote::{Remote, RemoteError},
    repos::ReposService,
    space_usage::SpaceUsageService,
    store,
    user::UserService,
};

use super::errors::LoadError;

pub struct LifecycleService {
    oauth2_service: Arc<OAuth2Service>,
    user_service: Arc<UserService>,
    repos_service: Arc<ReposService>,
    eventstream_service: Arc<EventStreamService>,
    space_usage_service: Arc<SpaceUsageService>,
    store: Arc<store::Store>,
}

impl LifecycleService {
    pub fn new(
        oauth2_service: Arc<OAuth2Service>,
        user_service: Arc<UserService>,
        repos_service: Arc<ReposService>,
        eventstream_service: Arc<EventStreamService>,
        space_usage_service: Arc<SpaceUsageService>,
        remote: Arc<Remote>,
        store: Arc<store::Store>,
    ) -> Arc<Self> {
        let lifecycle_service = Arc::new(Self {
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
                lifecycle_service.logout();
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

    pub fn logout(&self) {
        self.oauth2_service.logout();

        self.on_logout();
    }

    pub fn on_logout(&self) {
        self.eventstream_service.disconnect();

        self.store.mutate(|state, notify, _, _| {
            state.reset();

            for event in store::Event::all() {
                notify(event);
            }
        });

        self.repos_service.reset();

        // store state reset will set the oauth2 state status to loading, load
        // is called to set it to loaded
        let _ = self.oauth2_service.load();
    }
}
