use futures::FutureExt;

use vault_core::{
    remote_files_browsers::{self, state::RemoteFilesBrowserOptions},
    store,
    types::RemotePath,
};
use vault_core_tests::helpers::{eventstream::eventstream_wait_registered, with_user};

#[test]
fn test_eventstream() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let fixture1 = fixture.new_session();
            fixture1.login();
            fixture1.load().await;

            let (browser_id, load_future) = fixture.vault.remote_files_browsers_create(
                &remote_files_browsers::selectors::get_file_item_id(
                    &remote_files_browsers::selectors::ITEM_ID_PREFIX_PLACES,
                    &fixture.mount_id,
                    &RemotePath("/".into()),
                ),
                RemoteFilesBrowserOptions {
                    select_name: None,
                    only_hosted_devices: true,
                },
            );
            load_future.await.unwrap();
            eventstream_wait_registered(
                fixture.vault.store.clone(),
                &fixture.mount_id,
                &RemotePath("/".into()),
            )
            .await;

            fixture1.upload_remote_file("/file.txt", "test").await;

            {
                let store = fixture.vault.store.clone();
                let intl_service = fixture.vault.intl_service.clone();

                store::wait_for(
                    store.clone(),
                    &[store::Event::RemoteFilesBrowsers],
                    move |_| {
                        store.with_state(|state| {
                            remote_files_browsers::selectors::select_info(
                                state,
                                browser_id,
                                &intl_service,
                            )
                            .filter(|info| !info.items.is_empty())
                            .map(|_| ())
                        })
                    },
                )
                .await;
            }

            fixture.vault.remote_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_eventstream_not_loaded() {
    with_user(|fixture| {
        async move {
            let vault_load_future = fixture.vault.load().unwrap();

            let fixture1 = fixture.new_session();
            fixture1.login();
            fixture1.load().await;

            let (browser_id, load_future) = fixture.vault.remote_files_browsers_create(
                &remote_files_browsers::selectors::get_file_item_id(
                    &remote_files_browsers::selectors::ITEM_ID_PREFIX_PLACES,
                    &fixture.mount_id,
                    &RemotePath("/".into()),
                ),
                RemoteFilesBrowserOptions {
                    select_name: None,
                    only_hosted_devices: true,
                },
            );
            load_future.await.unwrap();

            vault_load_future.await.unwrap();

            eventstream_wait_registered(
                fixture.vault.store.clone(),
                &fixture.mount_id,
                &RemotePath("/".into()),
            )
            .await;

            {
                let store = fixture.vault.store.clone();
                let intl_service = fixture.vault.intl_service.clone();

                store::wait_for(
                    store.clone(),
                    &[store::Event::RemoteFilesBrowsers],
                    move |_| {
                        store.with_state(|state| {
                            remote_files_browsers::selectors::select_info(
                                state,
                                browser_id,
                                &intl_service,
                            )
                            .filter(|info| info.status.loaded())
                            .map(|_| ())
                        })
                    },
                )
                .await;
            }

            fixture1.upload_remote_file("/file.txt", "test").await;

            {
                let store = fixture.vault.store.clone();
                let intl_service = fixture.vault.intl_service.clone();

                store::wait_for(
                    store.clone(),
                    &[store::Event::RemoteFilesBrowsers],
                    move |_| {
                        store.with_state(|state| {
                            remote_files_browsers::selectors::select_info(
                                state,
                                browser_id,
                                &intl_service,
                            )
                            .filter(|info| !info.items.is_empty())
                            .map(|_| ())
                        })
                    },
                )
                .await;
            }

            fixture.vault.remote_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}
