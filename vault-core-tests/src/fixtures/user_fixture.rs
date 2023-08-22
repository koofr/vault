use std::sync::Arc;

use vault_core::{
    oauth2::{service::TOKEN_STORAGE_KEY, state::OAuth2Token},
    Vault,
};
use vault_fake_remote::fake_remote::{actions, context::Context, utils::now_ms};

use crate::fake_remote::FakeRemote;

use super::vault_fixture::VaultFixture;

pub struct UserFixture {
    pub vault_fixture: Arc<VaultFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub vault: Arc<Vault>,
    pub user_id: String,
    pub mount_id: String,
    pub oauth2_access_token: String,
    pub oauth2_refresh_token: String,
    pub context: Context,
}

impl UserFixture {
    pub fn create(vault_fixture: Arc<VaultFixture>) -> Arc<Self> {
        let vault = vault_fixture.vault.clone();
        let fake_remote = vault_fixture.fake_remote_fixture.fake_remote.clone();

        let (user_id, mount_id, oauth2_access_token, oauth2_refresh_token) = {
            let mut state = fake_remote.state.write().unwrap();

            let (user_id, mount_id) =
                actions::create_user(&mut state, &fake_remote.files_service, None, None);

            let oauth2_access_token = uuid::Uuid::new_v4().to_string();
            let oauth2_refresh_token = uuid::Uuid::new_v4().to_string();

            state
                .oauth2_access_tokens
                .insert(oauth2_access_token.clone(), user_id.clone());
            state
                .oauth2_refresh_tokens
                .insert(oauth2_refresh_token.clone(), user_id.clone());

            (user_id, mount_id, oauth2_access_token, oauth2_refresh_token)
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
                    expires_at: (now_ms() + 3600_000) as f64,
                },
            )
            .unwrap();
    }

    pub async fn load(&self) {
        self.vault_fixture.vault.load().await.unwrap();
    }
}
