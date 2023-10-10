use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core::{
    repo_files::{
        errors::LoadFilesError,
        state::{RepoFilesSort, RepoFilesSortField},
    },
    repo_files_browsers::state::{
        RepoFilesBrowserInfo, RepoFilesBrowserItem, RepoFilesBrowserOptions,
    },
    repos::errors::{RepoLockedError, RepoNotFoundError},
    selection::state::SelectionSummary,
    sort::state::SortDirection,
};
use vault_core_tests::helpers::with_repo;

#[test]
fn test_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                &fixture.repo_id,
                "/",
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let get_state = || fixture.vault.with_state(|state| state.clone());
            let select_info = |state| {
                vault_core::repo_files_browsers::selectors::select_info(state, browser_id).unwrap()
            };
            let select_items =
                |state| vault_core::repo_files_browsers::selectors::select_items(state, browser_id);

            let (_, file) = fixture.upload_file("/file.txt", "test").await;
            let dir = fixture.create_dir("/dir").await;

            let state_before_lock = get_state();
            assert_eq!(
                select_info(&state_before_lock),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some("/"),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: vault_core::common::state::Status::Loaded,
                    title: Some("My safe box".to_owned()),
                    total_count: 2,
                    total_size: 4,
                    selected_count: 0,
                    selected_size: 0,
                    selected_file: None,
                    can_download_selected: false,
                    can_copy_selected: false,
                    can_move_selected: false,
                    can_delete_selected: false,
                }
            );
            assert_eq!(
                select_items(&state_before_lock),
                vec![
                    RepoFilesBrowserItem {
                        file: &dir,
                        is_selected: false,
                    },
                    RepoFilesBrowserItem {
                        file: &file,
                        is_selected: false,
                    }
                ]
            );

            fixture.lock();

            let state_after_lock = get_state();
            assert_eq!(
                select_info(&state_after_lock),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some("/"),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: vault_core::common::state::Status::Error {
                        error: LoadFilesError::RepoLocked(RepoLockedError),
                        loaded: false
                    },
                    title: Some("My safe box".to_owned()),
                    total_count: 0,
                    total_size: 0,
                    selected_count: 0,
                    selected_size: 0,
                    selected_file: None,
                    can_download_selected: false,
                    can_copy_selected: false,
                    can_move_selected: false,
                    can_delete_selected: false,
                }
            );
            assert_eq!(select_items(&state_after_lock), vec![]);

            fixture.unlock().await;

            let state_after_unlock = get_state();
            assert_eq!(
                select_info(&state_after_unlock),
                select_info(&state_before_lock)
            );
            assert_eq!(
                select_items(&state_after_unlock),
                select_items(&state_before_lock)
            );

            fixture.remove().await;

            let state_after_remove = get_state();
            assert_eq!(
                select_info(&state_after_remove),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some("/"),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: vault_core::common::state::Status::Error {
                        error: LoadFilesError::RepoNotFound(RepoNotFoundError),
                        loaded: false
                    },
                    title: None,
                    total_count: 0,
                    total_size: 0,
                    selected_count: 0,
                    selected_size: 0,
                    selected_file: None,
                    can_download_selected: false,
                    can_copy_selected: false,
                    can_move_selected: false,
                    can_delete_selected: false,
                }
            );
            assert_eq!(select_items(&state_after_remove), vec![]);

            fixture.vault.repo_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}
