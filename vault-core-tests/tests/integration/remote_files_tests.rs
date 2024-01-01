use std::collections::HashMap;

use futures::{join, FutureExt};
use similar_asserts::assert_eq;
use vault_core::{
    remote::{remote::RemoteFileTagsSetConditions, RemoteFileMoveConditions},
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
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir1")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir1/dir12")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir1/dir12/file121.txt")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2")));
            assert!(!state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2/dir22")));

            let eventstream_subscription = eventstream_subscribe(
                fixture.vault.store.clone(),
                fixture.mount_id.clone(),
                RemotePath("/".into()),
                "test",
            )
            .await;

            let move_from_path = RemotePath("/dir1".into());
            let move_to_path = RemotePath("/dir2/dir22/dir222".into());
            let move_future = fixture.vault.remote.move_file(
                &fixture.mount_id,
                &move_from_path,
                &fixture.mount_id,
                &move_to_path,
                RemoteFileMoveConditions {
                    if_size: None,
                    if_modified: None,
                    if_hash: None,
                },
            );
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
            assert!(!state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir1")));
            assert!(!state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir1/dir12")));
            assert!(!state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir1/dir12/file121.txt")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2/dir22")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2/dir22/dir222")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2/dir22/dir222/dir12")));
            assert!(state
                .files
                .contains_key(&fixture.get_remote_file_id("/dir2/dir22/dir222/dir12/file121.txt")));
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

            assert!(state
                .files
                .get(&fixture.get_remote_file_id("/file.txt"))
                .unwrap()
                .tags
                .is_empty());

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
