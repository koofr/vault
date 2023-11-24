use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core::common::state::Status;
use vault_core_tests::{
    fixtures::{
        oauth2_fixture::OAuth2Fixture, user_fixture::UserFixture, vault_fixture::VaultFixture,
    },
    helpers::{secure_storage::TestSecureStorage, with_fake_remote, with_vault},
};

#[test]
fn test_load() {
    with_vault(|vault_fixture| {
        async move {
            vault_fixture.vault.load().await.unwrap();
        }
        .boxed()
    });
}

#[test]
fn test_load_get_item_error() {
    with_fake_remote(|fake_remote_fixture| {
        async move {
            let mut secure_storage = TestSecureStorage::wrap_memory();
            secure_storage.get_item_fn = Box::new(|_| Err("secure storage get error".into()));

            let vault_fixture =
                VaultFixture::create_with_options(fake_remote_fixture, Box::new(secure_storage));
            let user_fixture = UserFixture::create(vault_fixture.clone());
            let oauth2_fixture = OAuth2Fixture::create(user_fixture.clone());

            assert_eq!(
                vault_fixture.vault.load().await.unwrap_err().to_string(),
                "storage error: secure storage error: secure storage get error"
            );

            match oauth2_fixture.get_status() {
                Status::Error { error, .. } => assert_eq!(
                    error.to_string(),
                    "storage error: secure storage error: secure storage get error"
                ),
                status => panic!("expected error got {:?}", status),
            };
        }
        .boxed()
    });
}

#[test]
fn test_login() {
    with_vault(|vault_fixture| {
        async move {
            let user_fixture = UserFixture::create(vault_fixture.clone());
            let oauth2_fixture = OAuth2Fixture::create(user_fixture.clone());

            oauth2_fixture.login().await;

            assert!(vault_fixture
                .vault
                .with_state(|state| { state.user.user.is_some() }));
        }
        .boxed()
    });
}

#[test]
fn test_oauth2_logout() {
    with_vault(|vault_fixture| {
        async move {
            let user_fixture = UserFixture::create(vault_fixture.clone());
            let oauth2_fixture = OAuth2Fixture::create(user_fixture.clone());

            oauth2_fixture.login().await;

            oauth2_fixture.logout().await;

            assert!(vault_fixture
                .vault
                .with_state(|state| { state.user.user.is_none() }));
        }
        .boxed()
    });
}

#[test]
fn test_oauth2_logout_not_logged_in() {
    with_vault(|vault_fixture| {
        async move {
            let user_fixture = UserFixture::create(vault_fixture.clone());
            let oauth2_fixture = OAuth2Fixture::create(user_fixture.clone());

            vault_fixture.vault.load().await.unwrap();

            oauth2_fixture.logout().await;

            assert!(vault_fixture
                .vault
                .with_state(|state| { state.user.user.is_none() }));
        }
        .boxed()
    });
}

#[test]
fn test_oauth2_logout_clear_error() {
    with_fake_remote(|fake_remote_fixture| {
        async move {
            let mut secure_storage = TestSecureStorage::wrap_memory();
            secure_storage.clear_fn = Box::new(|| Err("secure storage clear error".into()));

            let vault_fixture =
                VaultFixture::create_with_options(fake_remote_fixture, Box::new(secure_storage));
            let user_fixture = UserFixture::create(vault_fixture.clone());
            let oauth2_fixture = OAuth2Fixture::create(user_fixture.clone());

            oauth2_fixture.login().await;

            assert_eq!(oauth2_fixture.get_status(), Status::Loaded);

            let url = oauth2_fixture
                .oauth2_request(vault_fixture.vault.oauth2_start_logout_flow().unwrap())
                .await;

            assert_eq!(
                vault_fixture
                    .vault
                    .oauth2_finish_flow_url(&url)
                    .await
                    .unwrap_err()
                    .to_string(),
                "secure storage error: secure storage clear error"
            );

            assert!(matches!(oauth2_fixture.get_status(), Status::Initial));
        }
        .boxed()
    });
}

#[test]
fn test_internal_logout_not_logged_in() {
    with_vault(|vault_fixture| {
        async move {
            vault_fixture.vault.load().await.unwrap();

            vault_fixture.vault.logout().unwrap();

            assert!(vault_fixture
                .vault
                .with_state(|state| { state.user.user.is_none() }));
        }
        .boxed()
    });
}
