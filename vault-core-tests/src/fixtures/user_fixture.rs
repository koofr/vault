use std::sync::Arc;

use futures::io::Cursor;
use vault_core::{
    oauth2::{service::TOKEN_STORAGE_KEY, state::OAuth2Token},
    remote::RemoteFileUploadConflictResolution,
    remote_files::state::RemoteFile,
    types::{MountId, RemotePath},
    utils::remote_path_utils,
    Vault,
};
use vault_fake_remote::fake_remote::{context::Context, utils::now_ms};

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

    pub fn login(&self) {
        self.vault
            .secure_storage_service
            .set(
                TOKEN_STORAGE_KEY,
                &OAuth2Token {
                    access_token: self.oauth2_access_token.clone(),
                    refresh_token: self.oauth2_refresh_token.clone(),
                    expires_at: (now_ms() + 3600_000) as f64,
                },
            )
            .unwrap();
    }

    pub async fn load(&self) {
        self.vault_fixture.vault.load().await.unwrap();
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
