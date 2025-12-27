use std::collections::HashMap;

use futures::{FutureExt, join};
use similar_asserts::assert_eq;

use vault_core::{
    remote::{RemoteFileMoveConditions, remote::RemoteFileTagsSetConditions},
    store,
    types::RemotePath,
};
use vault_core_tests::helpers::{eventstream::eventstream_subscribe, with_user};

#[test]
fn test_file_moved() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let get_state = || {
                fixture
                    .vault
                    .store
                    .with_state(|state| state.remote_files.clone())
            };

            fixture.create_remote_dir("/dir1").await;
            fixture.upload_remote_file("/dir1/file11.txt", "test").await;
            fixture.create_remote_dir("/dir1/dir12").await;
            fixture
                .upload_remote_file("/dir1/dir12/file121.txt", "test")
                .await;
            fixture.create_remote_dir("/dir2").await;
            fixture.create_remote_dir("/dir2/dir22").await;

            fixture.logout();
            fixture.login();
            fixture.load().await;

            fixture
                .vault
                .remote_files_service
                .load_files(&fixture.mount_id, &RemotePath("/".into()))
                .await
                .unwrap();
            fixture
                .vault
                .remote_files_service
                .load_files(&fixture.mount_id, &RemotePath("/dir1".into()))
                .await
                .unwrap();
            fixture
                .vault
                .remote_files_service
                .load_files(&fixture.mount_id, &RemotePath("/dir1/dir12".into()))
                .await
                .unwrap();

            let state = get_state();

            assert!(state.files.contains_key(&fixture.get_remote_file_id("/")));
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir1"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir1/dir12"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir1/dir12/file121.txt"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir2"))
            );
            assert!(
                !state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir2/dir22"))
            );

            let eventstream_subscription = eventstream_subscribe(
                fixture.vault.store.clone(),
                fixture.mount_id.clone(),
                RemotePath("/".into()),
                "test",
            )
            .await;

            let move_future = async {
                fixture
                    .vault
                    .remote
                    .move_file(
                        &fixture.mount_id,
                        &RemotePath("/dir1".into()),
                        &fixture.mount_id,
                        &RemotePath("/dir2/dir22/dir222".into()),
                        RemoteFileMoveConditions {
                            if_size: None,
                            if_modified: None,
                            if_hash: None,
                        },
                    )
                    .await
                    .unwrap()
            };
            let moved_future = store::wait_for(
                fixture.vault.store.clone(),
                &[store::Event::RemoteFiles],
                move |mutation_state| {
                    mutation_state
                        .filter(|state| !state.remote_files.moved_files.is_empty())
                        .map(|_| ())
                },
            );
            let _ = join!(move_future, moved_future);

            drop(eventstream_subscription);

            let state = get_state();

            assert!(state.files.contains_key(&fixture.get_remote_file_id("/")));
            assert!(
                !state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir1"))
            );
            assert!(
                !state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir1/dir12"))
            );
            assert!(
                !state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir1/dir12/file121.txt"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir2"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir2/dir22"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir2/dir22/dir222"))
            );
            assert!(
                state
                    .files
                    .contains_key(&fixture.get_remote_file_id("/dir2/dir22/dir222/dir12"))
            );
            assert!(
                state.files.contains_key(
                    &fixture.get_remote_file_id("/dir2/dir22/dir222/dir12/file121.txt")
                )
            );
        }
        .boxed()
    });
}

#[test]
fn test_set_tags() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let get_state = || {
                fixture
                    .vault
                    .store
                    .with_state(|state| state.remote_files.clone())
            };

            fixture.upload_remote_file("/file.txt", "test").await;

            fixture.logout();
            fixture.login();
            fixture.load().await;

            fixture
                .vault
                .remote_files_service
                .load_files(&fixture.mount_id, &RemotePath("/".into()))
                .await
                .unwrap();

            let state = get_state();

            assert!(
                state
                    .files
                    .get(&fixture.get_remote_file_id("/file.txt"))
                    .unwrap()
                    .tags
                    .is_empty()
            );

            let eventstream_subscription = eventstream_subscribe(
                fixture.vault.store.clone(),
                fixture.mount_id.clone(),
                RemotePath("/".into()),
                "test",
            )
            .await;

            let path = RemotePath("/file.txt".into());
            let set_tags_future = fixture.vault.remote_files_service.set_tags(
                &fixture.mount_id,
                &path,
                HashMap::from([("k1".into(), vec!["v1".into(), "v2".into()])]),
                RemoteFileTagsSetConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                    if_old_tags: None,
                },
            );
            let tags_updated_future = store::wait_for(
                fixture.vault.store.clone(),
                &[store::Event::RemoteFiles],
                move |mutation_state| {
                    mutation_state
                        .filter(|state| !state.remote_files.tags_updated.is_empty())
                        .map(|_| ())
                },
            );
            let _ = join!(set_tags_future, tags_updated_future);

            drop(eventstream_subscription);

            let state = get_state();

            assert_eq!(
                state
                    .files
                    .get(&fixture.get_remote_file_id("/file.txt"))
                    .unwrap()
                    .tags,
                HashMap::from([("k1".into(), vec!["v1".into(), "v2".into()])])
            );
        }
        .boxed()
    });
}

#[test]
fn test_load_recent() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let get_state = || {
                fixture.vault.store.with_state(|state| {
                    state
                        .remote_files
                        .recent
                        .iter()
                        .map(|(x, y)| (x.clone(), y.clone()))
                        .collect::<HashMap<_, _>>()
                })
            };

            fixture.upload_remote_file("/file.txt", "test").await;

            fixture
                .vault
                .remote_files_service
                .load_recent(&fixture.mount_id, &RemotePath("/".into()), 1000, None)
                .await
                .unwrap();

            assert_eq!(
                get_state(),
                HashMap::from([(
                    fixture.get_remote_file_id("/"),
                    vec![
                        fixture.get_remote_file_id("/file.txt"),
                        fixture.get_remote_file_id("/")
                    ]
                )])
            );
        }
        .boxed()
    });
}

#[test]
fn test_recent_file_deleted() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            fixture.create_remote_dir("/dir1").await;
            fixture.upload_remote_file("/dir1/file11.txt", "test").await;
            fixture.create_remote_dir("/dir1/dir12").await;
            fixture
                .upload_remote_file("/dir1/dir12/file121.txt", "test")
                .await;
            fixture.create_remote_dir("/dir12").await;
            fixture
                .upload_remote_file("/dir12/file121.txt", "test")
                .await;
            fixture.upload_remote_file("/file.txt", "test").await;

            fixture
                .vault
                .remote_files_service
                .load_recent(&fixture.mount_id, &RemotePath("/".into()), 1000, None)
                .await
                .unwrap();

            fixture
                .vault
                .remote_files_service
                .load_recent(&fixture.mount_id, &RemotePath("/dir1".into()), 1000, None)
                .await
                .unwrap();

            fixture
                .vault
                .remote_files_service
                .load_recent(
                    &fixture.mount_id,
                    &RemotePath("/dir1/dir12".into()),
                    1000,
                    None,
                )
                .await
                .unwrap();

            assert_eq!(
                fixture.get_remote_files_recent(),
                HashMap::from([
                    (
                        fixture.get_remote_file_id("/"),
                        vec![
                            fixture.get_remote_file_id("/file.txt"),
                            fixture.get_remote_file_id("/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12"),
                            fixture.get_remote_file_id("/dir1/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir1/dir12"),
                            fixture.get_remote_file_id("/dir1/file11.txt"),
                            fixture.get_remote_file_id("/dir1"),
                            fixture.get_remote_file_id("/"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir1"),
                        vec![
                            fixture.get_remote_file_id("/dir1/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir1/dir12"),
                            fixture.get_remote_file_id("/dir1/file11.txt"),
                            fixture.get_remote_file_id("/dir1"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir1/dir12"),
                        vec![
                            fixture.get_remote_file_id("/dir1/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir1/dir12"),
                        ]
                    ),
                ])
            );

            fixture
                .vault
                .remote_files_service
                .delete_file(&fixture.mount_id, &RemotePath("/dir1".into()))
                .await
                .unwrap();

            assert_eq!(
                fixture.get_remote_files_recent(),
                HashMap::from([(
                    fixture.get_remote_file_id("/"),
                    vec![
                        fixture.get_remote_file_id("/file.txt"),
                        fixture.get_remote_file_id("/dir12/file121.txt"),
                        fixture.get_remote_file_id("/dir12"),
                        fixture.get_remote_file_id("/"),
                    ]
                )])
            );
        }
        .boxed()
    });
}

#[test]
fn test_recent_file_moved() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            fixture.create_remote_dir("/dir1").await;
            fixture.upload_remote_file("/dir1/file11.txt", "test").await;
            fixture.create_remote_dir("/dir1/dir12").await;
            fixture
                .upload_remote_file("/dir1/dir12/file121.txt", "test")
                .await;
            fixture.create_remote_dir("/dir12").await;
            fixture
                .upload_remote_file("/dir12/file121.txt", "test")
                .await;

            fixture
                .vault
                .remote_files_service
                .load_recent(&fixture.mount_id, &RemotePath("/".into()), 1000, None)
                .await
                .unwrap();

            fixture
                .vault
                .remote_files_service
                .load_recent(&fixture.mount_id, &RemotePath("/dir1".into()), 1000, None)
                .await
                .unwrap();

            fixture
                .vault
                .remote_files_service
                .load_recent(
                    &fixture.mount_id,
                    &RemotePath("/dir1/dir12".into()),
                    1000,
                    None,
                )
                .await
                .unwrap();

            fixture
                .vault
                .remote_files_service
                .load_recent(&fixture.mount_id, &RemotePath("/dir12".into()), 1000, None)
                .await
                .unwrap();

            assert_eq!(
                fixture.get_remote_files_recent(),
                HashMap::from([
                    (
                        fixture.get_remote_file_id("/"),
                        vec![
                            fixture.get_remote_file_id("/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12"),
                            fixture.get_remote_file_id("/dir1/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir1/dir12"),
                            fixture.get_remote_file_id("/dir1/file11.txt"),
                            fixture.get_remote_file_id("/dir1"),
                            fixture.get_remote_file_id("/"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir1"),
                        vec![
                            fixture.get_remote_file_id("/dir1/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir1/dir12"),
                            fixture.get_remote_file_id("/dir1/file11.txt"),
                            fixture.get_remote_file_id("/dir1"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir1/dir12"),
                        vec![
                            fixture.get_remote_file_id("/dir1/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir1/dir12"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir12"),
                        vec![
                            fixture.get_remote_file_id("/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12"),
                        ]
                    ),
                ])
            );

            let eventstream_subscription = eventstream_subscribe(
                fixture.vault.store.clone(),
                fixture.mount_id.clone(),
                RemotePath("/".into()),
                "test",
            )
            .await;

            let move_future = async {
                fixture
                    .vault
                    .remote_files_service
                    .move_file(
                        &fixture.mount_id,
                        &RemotePath("/dir1".into()),
                        &fixture.mount_id,
                        &RemotePath("/dir12/dirx".into()),
                    )
                    .await
                    .unwrap()
            };
            let moved_future = store::wait_for(
                fixture.vault.store.clone(),
                &[store::Event::RemoteFiles],
                move |mutation_state| {
                    mutation_state
                        .filter(|state| !state.remote_files.moved_files.is_empty())
                        .map(|_| ())
                },
            );
            let _ = join!(move_future, moved_future);

            drop(eventstream_subscription);

            assert_eq!(
                fixture.get_remote_files_recent(),
                HashMap::from([
                    (
                        fixture.get_remote_file_id("/"),
                        vec![
                            fixture.get_remote_file_id("/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12"),
                            fixture.get_remote_file_id("/dir12/dirx/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12/dirx/dir12"),
                            fixture.get_remote_file_id("/dir12/dirx/file11.txt"),
                            fixture.get_remote_file_id("/dir12/dirx"),
                            fixture.get_remote_file_id("/"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir12"),
                        vec![
                            fixture.get_remote_file_id("/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir12/dirx"),
                        vec![
                            fixture.get_remote_file_id("/dir12/dirx/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12/dirx/dir12"),
                            fixture.get_remote_file_id("/dir12/dirx/file11.txt"),
                            fixture.get_remote_file_id("/dir12/dirx"),
                        ]
                    ),
                    (
                        fixture.get_remote_file_id("/dir12/dirx/dir12"),
                        vec![
                            fixture.get_remote_file_id("/dir12/dirx/dir12/file121.txt"),
                            fixture.get_remote_file_id("/dir12/dirx/dir12"),
                        ]
                    ),
                ])
            );
        }
        .boxed()
    });
}
