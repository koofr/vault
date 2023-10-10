use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core::repo_files::state::RepoFilesState;
use vault_core_tests::helpers::with_repo;

#[test]
fn test_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            let get_state = || fixture.vault.with_state(|state| state.repo_files.clone());

            let _ = fixture.upload_file("/file1.txt", "test").await;
            fixture.create_dir("/dir1").await;
            let _ = fixture.upload_file("/dir1/file11.txt", "test").await;
            fixture.create_dir("/dir1/dir12").await;
            let _ = fixture.upload_file("/dir1/dir12/file121.txt", "test").await;

            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, "/")
                .await
                .unwrap();
            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, "/dir1")
                .await
                .unwrap();
            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, "/dir1/dir12")
                .await
                .unwrap();

            let state_before_lock = get_state();

            fixture.lock();

            let state_after_lock = get_state();

            assert_eq!(state_after_lock, RepoFilesState::default());

            fixture.unlock().await;

            let state_after_unlock = get_state();

            assert_eq!(state_after_unlock, state_before_lock);

            fixture.remove().await;

            let state_after_remove = get_state();

            assert_eq!(state_after_remove, RepoFilesState::default());
        }
        .boxed()
    });
}
