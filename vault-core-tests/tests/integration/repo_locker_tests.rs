use std::time::Duration;

use futures::FutureExt;
use tokio::time::timeout;
use vault_core::{
    repos::{
        self,
        state::{RepoAutoLock, RepoAutoLockAfter},
    },
    store,
};
use vault_core_tests::{
    fixtures::repo_fixture::RepoFixture,
    helpers::{with_repo, with_user},
};

#[test]
fn test_lock_after() {
    with_user(|fixture| {
        async move {
            let fixture = RepoFixture::create(fixture).await;
            fixture.user_fixture.load().await;

            fixture.vault.store.mutate(|state, _, _, _| {
                state.config.repo_locker.lock_check_interval = Duration::from_millis(20);
            });

            fixture
                .vault
                .repos_set_auto_lock(
                    &fixture.repo_id,
                    RepoAutoLock {
                        after: Some(RepoAutoLockAfter::Custom(Duration::from_millis(100))),
                        on_app_hidden: false,
                    },
                )
                .unwrap();

            let is_locked = || {
                fixture.vault.store.with_state(|state| {
                    repos::selectors::select_repo(state, &fixture.repo_id)
                        .unwrap()
                        .state
                        .is_locked()
                })
            };

            fixture.unlock();

            for _ in 0..5 {
                let _ = fixture.vault.repos_touch_repo(&fixture.repo_id);

                assert!(!is_locked());

                fixture.vault.runtime.sleep(Duration::from_millis(50)).await;
            }

            // reload repos to test if auto_lock and last_activity are retained
            fixture.vault.load().unwrap().await.unwrap();

            let wait_store = fixture.vault.store.clone();
            timeout(
                Duration::from_millis(1000),
                store::wait_for(wait_store.clone(), &[store::Event::Repos], move |_| {
                    if wait_store.with_state(|state| {
                        repos::selectors::select_repo(state, &fixture.repo_id)
                            .unwrap()
                            .state
                            .is_locked()
                    }) {
                        Some(())
                    } else {
                        None
                    }
                }),
            )
            .await
            .unwrap();
        }
        .boxed()
    });
}

#[test]
fn test_lock_after_set_already_unlocked() {
    with_user(|fixture| {
        async move {
            let fixture = RepoFixture::create(fixture).await;
            fixture.user_fixture.load().await;

            fixture.vault.store.mutate(|state, _, _, _| {
                state.config.repo_locker.lock_check_interval = Duration::from_millis(20);
            });

            fixture.unlock();

            fixture.vault.runtime.sleep(Duration::from_millis(30)).await;

            fixture
                .vault
                .repos_set_auto_lock(
                    &fixture.repo_id,
                    RepoAutoLock {
                        after: Some(RepoAutoLockAfter::Custom(Duration::from_millis(100))),
                        on_app_hidden: false,
                    },
                )
                .unwrap();

            let is_locked = || {
                fixture.vault.store.with_state(|state| {
                    repos::selectors::select_repo(state, &fixture.repo_id)
                        .unwrap()
                        .state
                        .is_locked()
                })
            };

            for _ in 0..5 {
                let _ = fixture.vault.repos_touch_repo(&fixture.repo_id);

                assert!(!is_locked());

                fixture.vault.runtime.sleep(Duration::from_millis(50)).await;
            }

            // reload repos to test if auto_lock and last_activity are retained
            fixture.vault.load().unwrap().await.unwrap();

            let wait_store = fixture.vault.store.clone();
            timeout(
                Duration::from_millis(1000),
                store::wait_for(wait_store.clone(), &[store::Event::Repos], move |_| {
                    if wait_store.with_state(|state| {
                        repos::selectors::select_repo(state, &fixture.repo_id)
                            .unwrap()
                            .state
                            .is_locked()
                    }) {
                        Some(())
                    } else {
                        None
                    }
                }),
            )
            .await
            .unwrap();
        }
        .boxed()
    });
}

#[test]
fn test_lock_on_app_hidden() {
    with_repo(|fixture| {
        async move {
            fixture
                .vault
                .repos_set_auto_lock(
                    &fixture.repo_id,
                    RepoAutoLock {
                        after: None,
                        on_app_hidden: true,
                    },
                )
                .unwrap();

            // reload repos to test if auto_lock is retained
            fixture.vault.load().unwrap().await.unwrap();

            let is_locked = || {
                fixture.vault.store.with_state(|state| {
                    repos::selectors::select_repo(state, &fixture.repo_id)
                        .unwrap()
                        .state
                        .is_locked()
                })
            };

            assert!(!is_locked());

            fixture.vault.app_hidden();

            assert!(is_locked());

            fixture.vault.app_visible();

            assert!(is_locked());
        }
        .boxed()
    });
}
