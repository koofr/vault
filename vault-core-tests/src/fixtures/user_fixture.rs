use std::{sync::Arc, time::Duration};

use futures::io::Cursor;
use vault_core::{
    oauth2::{service::TOKEN_STORAGE_KEY, state::OAuth2Token},
    remote::RemoteFileUploadConflictResolution,
    remote_files::{self, state::RemoteFile},
    types::{MountId, RemoteFileId, RemotePath},
    utils::remote_path_utils,
    Vault,
};
use vault_fake_remote::fake_remote::context::Context;
use vault_native::native_runtime::now;

use crate::fake_remote::FakeRemote;

use super::vault_fixture::VaultFixture;

pub struct UserFixture {
    pub vault_fixture: Arc<VaultFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub vault: Arc<Vault>,
    pub user_id: String,
    pub mount_id: MountId,
    pub oauth2_access_token: String,
    pub oauth2_refresh_token: String,
    pub context: Context,
}

impl UserFixture {
    pub fn create(vault_fixture: Arc<VaultFixture>) -> Arc<Self> {
        let vault = vault_fixture.vault.clone();
        let fake_remote = vault_fixture.fake_remote_fixture.fake_remote.clone();

        let (user_id, mount_id) = fake_remote.app_state.users_service.create_user(None, None);

        let oauth2_access_token = uuid::Uuid::new_v4().to_string();
        let oauth2_refresh_token = uuid::Uuid::new_v4().to_string();

        {
            let mut state = fake_remote.app_state.state.write().unwrap();

            state
                .oauth2_access_tokens
                .insert(oauth2_access_token.clone(), user_id.clone());
            state
                .oauth2_refresh_tokens
                .insert(oauth2_refresh_token.clone(), user_id.clone());
        };

        let context = Context {
            user_id: user_id.clone(),
            user_agent: None,
        };

        Arc::new(Self {
            vault_fixture,

            fake_remote,
            vault,
            user_id,
            mount_id: MountId(mount_id),
            oauth2_access_token,
            oauth2_refresh_token,
            context,
        })
    }

    pub fn new_session(&self) -> Arc<Self> {
        let vault_fixture = self.vault_fixture.new_session();
        let fake_remote = vault_fixture.fake_remote_fixture.fake_remote.clone();
        let vault = vault_fixture.vault.clone();

        let user_id = self.user_id.clone();
        let mount_id = self.mount_id.clone();
        let oauth2_access_token = self.oauth2_access_token.clone();
        let oauth2_refresh_token = self.oauth2_refresh_token.clone();
        let context = self.context.clone();

        Arc::new(Self {
            vault_fixture,

            fake_remote,
            vault,
            user_id,
            mount_id,
            oauth2_access_token,
            oauth2_refresh_token,
            context,
        })
    }

    pub fn login(&self) {
        self.vault
            .secure_storage_service
            .set(
                TOKEN_STORAGE_KEY,
                &OAuth2Token {
                    access_token: self.oauth2_access_token.clone(),
                    refresh_token: self.oauth2_refresh_token.clone(),
                    expires_at: now() + Duration::from_secs(3600),
                },
            )
            .unwrap();
    }

    pub fn logout(&self) {
        self.vault.logout().unwrap();
    }

    pub async fn load(&self) {
        self.vault.load().await.unwrap();
    }

    pub fn get_remote_file_id(&self, path: &str) -> RemoteFileId {
        remote_files::selectors::get_file_id(
            &self.mount_id,
            &RemotePath(path.into()).to_lowercase(),
        )
    }

    pub async fn create_remote_dir(&self, path: &str) -> RemoteFile {
        let path = RemotePath(path.to_owned());
        let (parent_path, name) = remote_path_utils::split_parent_name(&path).unwrap();

        self.vault
            .remote_files_service
            .clone()
            .create_dir_name(&self.mount_id, &parent_path, name)
            .await
            .unwrap();

        self.vault.with_state(|state| {
            state
                .remote_files
                .files
                .get(&remote_files::selectors::get_file_id(
                    &self.mount_id,
                    &path.to_lowercase(),
                ))
                .cloned()
                .unwrap()
        })
    }

    pub async fn upload_remote_file(&self, path: &str, content: &str) -> RemoteFile {
        let path = RemotePath(path.to_owned());
        let (parent_path, name) = remote_path_utils::split_parent_name(&path).unwrap();

        let bytes = content.as_bytes().to_vec();
        let size = bytes.len();
        let reader = Box::pin(Cursor::new(bytes));

        let (_, remote_file) = self
            .vault
            .remote_files_service
            .clone()
            .upload_file_reader(
                &self.mount_id,
                &parent_path,
                &name,
                reader,
                Some(size as i64),
                RemoteFileUploadConflictResolution::Error,
                None,
            )
            .await
            .unwrap();

        remote_file
    }
}
