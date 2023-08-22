use std::{future::Future, sync::Arc};

use crate::fixtures::{
    fake_remote_fixture::FakeRemoteFixture, user_fixture::UserFixture, vault_fixture::VaultFixture,
};

pub fn with_vault<F: Future<Output = ()>>(f: impl FnOnce(Arc<UserFixture>) -> F) {
    let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    tokio_runtime.clone().block_on(async move {
        let fake_remote_fixture = FakeRemoteFixture::create(tokio_runtime).await;
        let vault_fixture = VaultFixture::create(fake_remote_fixture);
        let user_fixture = UserFixture::create(vault_fixture);

        user_fixture.login();

        f(user_fixture).await;
    });
}
