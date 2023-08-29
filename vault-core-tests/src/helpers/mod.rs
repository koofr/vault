pub mod transfers;

use std::{future::Future, sync::Arc};

use futures::{future::BoxFuture, FutureExt};

use crate::fixtures::{
    fake_remote_fixture::FakeRemoteFixture, repo_fixture::RepoFixture, user_fixture::UserFixture,
    vault_fixture::VaultFixture,
};

pub fn with_tokio_runtime(
    f: impl FnOnce(Arc<tokio::runtime::Runtime>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    let tokio_runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

    tokio_runtime.clone().block_on(
        async move {
            f(tokio_runtime).await;
        }
        .boxed(),
    );
}

pub fn with_fake_remote(
    f: impl FnOnce(Arc<FakeRemoteFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_tokio_runtime(|tokio_runtime| {
        async move {
            let fake_remote_fixture = FakeRemoteFixture::create(tokio_runtime).await;

            f(fake_remote_fixture).await;
        }
        .boxed()
    });
}

pub fn with_vault(
    f: impl FnOnce(Arc<VaultFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_fake_remote(|fake_remote_fixture| {
        async move {
            let vault_fixture = VaultFixture::create(fake_remote_fixture);

            f(vault_fixture).await;
        }
        .boxed()
    });
}

pub fn with_user(
    f: impl FnOnce(Arc<UserFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_vault(|vault_fixture| {
        async move {
            let user_fixture = UserFixture::create(vault_fixture);

            user_fixture.login();

            f(user_fixture).await;
        }
        .boxed()
    });
}

pub fn with_repo(
    f: impl FnOnce(Arc<RepoFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_user(|user_fixture| {
        async move {
            let repo_fixture = RepoFixture::create(user_fixture.clone()).await;

            user_fixture.load().await;

            repo_fixture.unlock().await;

            f(repo_fixture).await;
        }
        .boxed()
    });
}
