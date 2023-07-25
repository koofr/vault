use std::sync::Arc;

use futures::{future::BoxFuture, FutureExt};
use vault_core_tests::helpers::{wait_for_async, with_fake_remote};

use crate::fixtures::{
    mobile_vault_fixture::MobileVaultFixture, repo_fixture::RepoFixture, user_fixture::UserFixture,
};

pub fn with_mobile_vault(
    f: impl FnOnce(Arc<MobileVaultFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_fake_remote(|fake_remote_fixture| {
        async move {
            let mobile_vault_fixture = MobileVaultFixture::create(fake_remote_fixture);

            let store_weak = Arc::downgrade(&mobile_vault_fixture.mobile_vault.vault.store);
            let runtime_weak = Arc::downgrade(&mobile_vault_fixture.mobile_vault.vault.runtime);

            mobile_vault_fixture.load().await;

            f(mobile_vault_fixture).await;

            assert!(
                wait_for_async(500, move || store_weak.strong_count() == 0).await,
                "Store not dropped in 500 ms"
            );

            assert!(
                wait_for_async(500, move || runtime_weak.strong_count() == 0).await,
                "Runtime not dropped in 500 ms"
            );
        }
        .boxed()
    });
}

pub fn with_user(
    f: impl FnOnce(Arc<UserFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_mobile_vault(|mobile_vault_fixture| {
        async move {
            let user_fixture = UserFixture::create(mobile_vault_fixture);

            user_fixture.login().await;

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

            repo_fixture.unlock().await;

            f(repo_fixture).await;
        }
        .boxed()
    });
}
