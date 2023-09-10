use std::sync::{Arc, RwLock};

use vault_core::remote::models;

use super::{
    files::service::FilesService,
    state::{FakeRemoteState, UserContainer},
};

pub struct UsersService {
    state: Arc<RwLock<FakeRemoteState>>,
    files_service: Arc<FilesService>,
}

impl UsersService {
    pub fn new(state: Arc<RwLock<FakeRemoteState>>, files_service: Arc<FilesService>) -> Self {
        Self {
            state,
            files_service,
        }
    }

    pub fn create_user(
        &self,
        user_id: Option<String>,
        mount_id: Option<String>,
    ) -> (String, String) {
        let mut state = self.state.write().unwrap();

        let user_id = user_id.unwrap_or(uuid::Uuid::new_v4().to_string());

        let user: models::User = models::User {
            id: user_id.clone(),
            first_name: "Vault".into(),
            last_name: "Test".into(),
            email: user_id.replace("-", "") + "@example.com",
            phone_number: None,
            has_password: true,
            level: 1000,
        };

        let mount_id = mount_id.unwrap_or(uuid::Uuid::new_v4().to_string());

        let mount = models::Mount {
            id: mount_id.clone(),
            name: "Koofr".into(),
            typ: "device".into(),
            origin: "hosted".into(),
            online: true,
            is_primary: true,
            space_total: Some(10240),
            space_used: Some(0),
        };

        state.mounts.insert(mount_id.clone(), mount);

        let fs = self.files_service.create_filesystem();

        state.filesystems.insert(mount_id.clone(), fs);

        state.users.insert(
            user_id.clone(),
            UserContainer {
                user,
                mounts: vec![mount_id.clone()],
                user_vault_repos: vec![],
            },
        );

        (user_id, mount_id)
    }
}
