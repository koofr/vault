use std::sync::{Arc, RwLock};

use super::{eventstream, files::service::FilesService, state::FakeRemoteState};

#[derive(Clone)]
pub struct AppState {
    pub state: Arc<RwLock<FakeRemoteState>>,
    pub files_service: Arc<FilesService>,
    pub eventstream_listeners: Arc<eventstream::Listeners>,
}
