use std::sync::{Arc, RwLock};

use super::{
    eventstream,
    files::{objects::object_provider::BoxObjectProvider, service::FilesService},
    interceptor::Interceptor,
    state::FakeRemoteState,
    users_service::UsersService,
    vault_repos_service::{VaultReposCreateService, VaultReposRemoveService},
};

#[derive(Clone)]
pub struct AppState {
    pub state: Arc<RwLock<FakeRemoteState>>,
    pub object_provider: Arc<BoxObjectProvider>,
    pub files_service: Arc<FilesService>,
    pub users_service: Arc<UsersService>,
    pub vault_repos_create_service: Arc<VaultReposCreateService>,
    pub vault_repos_remove_service: Arc<VaultReposRemoveService>,
    pub eventstream_listeners: Arc<eventstream::Listeners>,
    pub interceptor: Arc<Option<Interceptor>>,
}

impl AppState {
    pub fn new(object_provider: Arc<BoxObjectProvider>) -> Self {
        let state = Arc::new(RwLock::new(FakeRemoteState::default()));
        let eventstream_listeners = Arc::new(eventstream::Listeners::new());
        let vault_repos_remove_service = Arc::new(VaultReposRemoveService::new(state.clone()));
        let files_service = Arc::new(FilesService::new(
            state.clone(),
            vault_repos_remove_service.clone(),
            eventstream_listeners.clone(),
            object_provider.clone(),
        ));
        let users_service = Arc::new(UsersService::new(state.clone(), files_service.clone()));
        let vault_repos_create_service = Arc::new(VaultReposCreateService::new(
            state.clone(),
            files_service.clone(),
        ));

        Self {
            state,
            object_provider,
            files_service,
            users_service,
            vault_repos_create_service,
            vault_repos_remove_service,
            eventstream_listeners,
            interceptor: Default::default(),
        }
    }
}
