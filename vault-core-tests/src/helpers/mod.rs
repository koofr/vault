use std::{future::Future, sync::Arc};

use crate::fixtures::{
    fake_remote_fixture::FakeRemoteFixture, repo_fixture::RepoFixture, user_fixture::UserFixture,
    vault_fixture::VaultFixture,
};

pub fn with_tokio_runtime<F: Future<Output = ()>>(
    f: impl FnOnce(Arc<tokio::runtime::Runtime>) -> F,
) {
    let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    tokio_runtime.clone().block_on(async move {
        f(tokio_runtime).await;
    });
}

pub fn with_vault<F: Future<Output = ()>>(f: impl FnOnce(Arc<VaultFixture>) -> F) {
    with_tokio_runtime(|tokio_runtime| async move {
        let fake_remote_fixture = FakeRemoteFixture::create(tokio_runtime).await;
        let vault_fixture = VaultFixture::create(fake_remote_fixture);

        f(vault_fixture).await;
    });
}

pub fn with_user<F: Future<Output = ()>>(f: impl FnOnce(Arc<UserFixture>) -> F) {
    with_vault(|vault_fixture| async move {
        let user_fixture = UserFixture::create(vault_fixture);

        user_fixture.login();

        f(user_fixture).await;
    });
}

pub fn with_repo<F: Future<Output = ()>>(f: impl FnOnce(Arc<RepoFixture>) -> F) {
    with_user(|user_fixture| async move {
        let repo_fixture = RepoFixture::create(user_fixture.clone()).await;

        user_fixture.load().await;

        repo_fixture.unlock().await;

        f(repo_fixture).await;
    });
}
