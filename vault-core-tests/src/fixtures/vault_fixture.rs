use std::sync::Arc;

use vault_core::{
    oauth2::OAuth2Config,
    secure_storage::{MemorySecureStorage, SecureStorage},
    Vault,
};
use vault_native::vault::build_vault;

use super::fake_remote_fixture::FakeRemoteFixture;

pub struct VaultFixture {
    pub fake_remote_fixture: Arc<FakeRemoteFixture>,

    pub vault: Arc<Vault>,
}

impl VaultFixture {
    pub fn create(fake_remote_fixture: Arc<FakeRemoteFixture>) -> Arc<Self> {
        let secure_storage = Box::new(MemorySecureStorage::new());

        Self::create_with_options(fake_remote_fixture, secure_storage)
    }

    pub fn create_with_options(
        fake_remote_fixture: Arc<FakeRemoteFixture>,
        secure_storage: Box<dyn SecureStorage + Send + Sync>,
    ) -> Arc<Self> {
        let oauth2_config = OAuth2Config {
            base_url: fake_remote_fixture.base_url.clone(),
            auth_base_url: fake_remote_fixture.base_url.clone(),
            client_id: "7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3".into(),
            client_secret: "VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB"
                .into(),
            redirect_uri: "http://127.0.0.1:5173/oauth2callback".into(),
        };

        let (vault, _, _) = build_vault(
            fake_remote_fixture.base_url.clone(),
            oauth2_config,
            secure_storage,
            fake_remote_fixture.tokio_runtime.clone(),
        );

        Arc::new(Self {
            fake_remote_fixture,

            vault,
        })
    }
}
