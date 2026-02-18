use futures::FutureExt;
use similar_asserts::assert_eq;

use vault_core::{
    intl::{
        IntlConfig, IntlConfigOwnership, service::CURRENT_LOCALE_STORAGE_KEY,
        state::ChangeLocaleStrategy,
    },
    secure_storage::MemorySecureStorage,
};
use vault_core_tests::helpers::with_vault_options;

#[test]
fn test_intl_ownership_core_preferred_locales_load_change_locale_logout_load() {
    let preferred_locales = vec!["sl-SI".parse().unwrap(), "en".parse().unwrap()];
    let secure_storage = MemorySecureStorage::new();

    with_vault_options(
        IntlConfig {
            ownership: IntlConfigOwnership::Core {
                preferred_locales: preferred_locales.clone(),
            },
        },
        Box::new(secure_storage.clone()),
        move |vault_fixture| {
            let secure_storage = secure_storage.clone();

            async move {
                // current locale is negotiated using the preferred locales list

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(
                        state.intl.ownership,
                        vault_core::intl::state::Ownership::Core { preferred_locales }
                    );
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );

                // current_locale is not overridden after load because current
                // locale is not saved in secure storage

                vault_fixture.vault.load().unwrap().await.unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );

                // intl_change_locale changes current locale and saves it to
                // secure storage

                vault_fixture
                    .vault
                    .intl_change_locale(ChangeLocaleStrategy::Exact("en".parse().unwrap()))
                    .unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("en".parse().unwrap()));
                });
                assert_eq!(
                    secure_storage
                        .get_data()
                        .get(CURRENT_LOCALE_STORAGE_KEY)
                        .cloned(),
                    Some("\"en\"".to_string())
                );

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Path is not valid"
                );

                // after logout, current locale is negotiated using the
                // preferred locales list, logout clears secure storage,
                // including current locale

                vault_fixture.vault.logout().unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });
                assert!(
                    secure_storage
                        .get_data()
                        .get(CURRENT_LOCALE_STORAGE_KEY)
                        .is_none()
                );

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );

                // current_locale is not overridden after load because secure
                // storage is cleared

                vault_fixture.vault.load().unwrap().await.unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );
            }
            .boxed()
        },
    );
}

#[test]
fn test_intl_ownership_core_preferred_locales_load_change_locale_new_vault_load() {
    let preferred_locales = vec!["sl-SI".parse().unwrap(), "en".parse().unwrap()];
    let secure_storage = MemorySecureStorage::new();

    with_vault_options(
        IntlConfig {
            ownership: IntlConfigOwnership::Core {
                preferred_locales: preferred_locales.clone(),
            },
        },
        Box::new(secure_storage.clone()),
        {
            let preferred_locales = preferred_locales.clone();

            |vault_fixture| {
                async move {
                    // current locale is negotiated using the preferred locales
                    // list

                    vault_fixture.vault.with_state(|state| {
                        assert_eq!(
                            state.intl.ownership,
                            vault_core::intl::state::Ownership::Core { preferred_locales }
                        );
                        assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                    });

                    assert_eq!(
                        &vault_fixture
                            .vault
                            .intl_service
                            .format_message("core.common.invalid_path.error", &[]),
                        "Pot ni veljavna"
                    );

                    // current_locale is not overridden after load because current
                    // locale is not saved in secure storage

                    vault_fixture.vault.load().unwrap().await.unwrap();

                    vault_fixture.vault.with_state(|state| {
                        assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                    });

                    assert_eq!(
                        &vault_fixture
                            .vault
                            .intl_service
                            .format_message("core.common.invalid_path.error", &[]),
                        "Pot ni veljavna"
                    );

                    // intl_change_locale changes current locale and saves it to
                    // secure storage

                    vault_fixture
                        .vault
                        .intl_change_locale(ChangeLocaleStrategy::Exact("en".parse().unwrap()))
                        .unwrap();

                    vault_fixture.vault.with_state(|state| {
                        assert_eq!(state.intl.current_locale, Some("en".parse().unwrap()));
                    });

                    assert_eq!(
                        &vault_fixture
                            .vault
                            .intl_service
                            .format_message("core.common.invalid_path.error", &[]),
                        "Path is not valid"
                    );
                }
                .boxed()
            }
        },
    );

    with_vault_options(
        IntlConfig {
            ownership: IntlConfigOwnership::Core {
                preferred_locales: preferred_locales.clone(),
            },
        },
        Box::new(secure_storage.clone()),
        |vault_fixture| {
            async move {
                // current locale is negotiated using the preferred locales
                // list

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(
                        state.intl.ownership,
                        vault_core::intl::state::Ownership::Core { preferred_locales }
                    );
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );

                // current_locale is overridden after load because current
                // locale is saved in secure storage

                vault_fixture.vault.load().unwrap().await.unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("en".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Path is not valid"
                );
            }
            .boxed()
        },
    );
}

#[test]
fn test_intl_ownership_external_load_change_locale_logout_load() {
    let secure_storage = MemorySecureStorage::new();

    with_vault_options(
        IntlConfig {
            ownership: IntlConfigOwnership::External,
        },
        Box::new(secure_storage.clone()),
        move |vault_fixture| {
            let secure_storage = secure_storage.clone();

            async move {
                // external ownership starts without a negotiated locale

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(
                        state.intl.ownership,
                        vault_core::intl::state::Ownership::External
                    );
                    assert_eq!(state.intl.current_locale, None);
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Path is not valid"
                );

                // load does not change locale in external ownership

                vault_fixture.vault.load().unwrap().await.unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, None);
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Path is not valid"
                );

                // intl_change_locale changes current locale in memory only

                vault_fixture
                    .vault
                    .intl_change_locale(ChangeLocaleStrategy::Exact("sl".parse().unwrap()))
                    .unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });
                assert!(
                    secure_storage
                        .get_data()
                        .get(CURRENT_LOCALE_STORAGE_KEY)
                        .is_none()
                );

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );

                // after logout with external ownership, current locale is kept

                vault_fixture.vault.logout().unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });
                assert_eq!(
                    secure_storage
                        .get_data()
                        .get(CURRENT_LOCALE_STORAGE_KEY)
                        .cloned(),
                    None
                );

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );

                // load remains a no-op for locale in external ownership

                vault_fixture.vault.load().unwrap().await.unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );
            }
            .boxed()
        },
    );
}

#[test]
fn test_intl_ownership_external_load_change_locale_new_vault_load() {
    let secure_storage = MemorySecureStorage::new();

    with_vault_options(
        IntlConfig {
            ownership: IntlConfigOwnership::External,
        },
        Box::new(secure_storage.clone()),
        |vault_fixture| {
            async move {
                vault_fixture.vault.with_state(|state| {
                    assert_eq!(
                        state.intl.ownership,
                        vault_core::intl::state::Ownership::External
                    );
                    assert_eq!(state.intl.current_locale, None);
                });

                vault_fixture
                    .vault
                    .intl_change_locale(ChangeLocaleStrategy::Exact("sl".parse().unwrap()))
                    .unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Pot ni veljavna"
                );
            }
            .boxed()
        },
    );

    with_vault_options(
        IntlConfig {
            ownership: IntlConfigOwnership::External,
        },
        Box::new(secure_storage.clone()),
        |vault_fixture| {
            async move {
                // locale from previous instance is not restored from storage

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(
                        state.intl.ownership,
                        vault_core::intl::state::Ownership::External
                    );
                    assert_eq!(state.intl.current_locale, None);
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Path is not valid"
                );

                vault_fixture.vault.load().unwrap().await.unwrap();

                vault_fixture.vault.with_state(|state| {
                    assert_eq!(state.intl.current_locale, None);
                });

                assert_eq!(
                    &vault_fixture
                        .vault
                        .intl_service
                        .format_message("core.common.invalid_path.error", &[]),
                    "Path is not valid"
                );
            }
            .boxed()
        },
    );
}
