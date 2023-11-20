use futures::{join, FutureExt};
use vault_core::{remote::RemoteFileMoveConditions, remote_files, store, types::RemotePath};
use vault_core_tests::helpers::with_user;

#[test]
fn test_file_moved() {
    with_user(|fixture| {
        async move {
            fixture.login();
            fixture.load().await;

            let get_state = || {
                fixture
                    .vault
                    .store
                    .with_state(|state| state.remote_files.clone())
            };
            let get_file_id = |path: &str| {
                remote_files::selectors::get_file_id(
                    &fixture.mount_id,
                    &RemotePath(path.into()).to_lowercase(),
                )
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

            assert!(state.files.contains_key(&get_file_id("/")));
            assert!(state.files.contains_key(&get_file_id("/dir1")));
            assert!(state.files.contains_key(&get_file_id("/dir1/dir12")));
            assert!(state
                .files
                .contains_key(&get_file_id("/dir1/dir12/file121.txt")));
            assert!(state.files.contains_key(&get_file_id("/dir2")));
            assert!(!state.files.contains_key(&get_file_id("/dir2/dir22")));

            let mount_subscription = fixture
                .vault
                .eventstream_service
                .clone()
                .get_mount_subscription(&fixture.mount_id, &RemotePath("".into()));

            // wait for mount subscription registration
            fixture
                .vault
                .runtime
                .sleep(std::time::Duration::from_millis(100))
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

            drop(mount_subscription);

            let state = get_state();

            assert!(state.files.contains_key(&get_file_id("/")));
            assert!(!state.files.contains_key(&get_file_id("/dir1")));
            assert!(!state.files.contains_key(&get_file_id("/dir1/dir12")));
            assert!(!state
                .files
                .contains_key(&get_file_id("/dir1/dir12/file121.txt")));
            assert!(state.files.contains_key(&get_file_id("/dir2")));
            assert!(state.files.contains_key(&get_file_id("/dir2/dir22")));
            assert!(state.files.contains_key(&get_file_id("/dir2/dir22/dir222")));
            assert!(state
                .files
                .contains_key(&get_file_id("/dir2/dir22/dir222/dir12")));
            assert!(state
                .files
                .contains_key(&get_file_id("/dir2/dir22/dir222/dir12/file121.txt")));
        }
        .boxed()
    });
}
