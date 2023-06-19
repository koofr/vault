use std::sync::Arc;

use crate::{
    eventstream::EventStreamService, oauth2::OAuth2Service, remote::RemoteError,
    repos::ReposService, space_usage::SpaceUsageService, store, user::UserService,
};

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
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            oauth2_service,
            user_service,
            repos_service,
            eventstream_service,
            space_usage_service,
            store,
        }
    }

    pub async fn load(&self) -> Result<(), RemoteError> {
        let _ = self.oauth2_service.load();

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
        self.oauth2_service.reset();

        self.on_logout();

        let _ = self.oauth2_service.load();
    }

    fn on_logout(&self) {
        self.eventstream_service.disconnect();

        self.store.mutate(|state, notify, _, _| {
            state.reset();

            for event in store::Event::all() {
                notify(event);
            }
        });

        self.repos_service.reset();
    }
}
