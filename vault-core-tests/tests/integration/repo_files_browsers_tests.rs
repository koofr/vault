use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core::{
    common::state::Status,
    repo_files::{
        errors::LoadFilesError,
        state::{RepoFilesSort, RepoFilesSortField},
    },
    repo_files_browsers::state::{
        RepoFilesBrowser, RepoFilesBrowserInfo, RepoFilesBrowserItem, RepoFilesBrowserLocation,
        RepoFilesBrowserOptions, RepoFilesBrowsersState,
    },
    repos::errors::{RepoLockedError, RepoNotFoundError},
    selection::state::SelectionSummary,
    sort::state::SortDirection,
    store,
};
use vault_core_tests::{fixtures::repo_fixture::RepoFixture, helpers::with_repo};
use vault_store::{test_helpers::StateRecorder, NextId};

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

#[test]
fn test_create() {
    with_repo(|fixture| {
        async move {
            let (_, file) = fixture.upload_file("/file.txt", "test").await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.repo_files_browsers.clone(),
            );

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                &fixture.repo_id,
                "/",
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            fixture.vault.repo_files_browsers_destroy(browser_id);

            recorder.check_recorded(
                |len| assert_eq!(len, 5),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesBrowsersState::default()),
                    1 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loading { loaded: false };
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loading { loaded: false };
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loaded;
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        RepoFilesBrowsersState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    })
}

#[test]
fn test_create_already_loaded() {
    with_repo(|fixture| {
        async move {
            let (_, file) = fixture.upload_file("/file.txt", "test").await;

            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, "/")
                .await
                .unwrap();

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.repo_files_browsers.clone(),
            );

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                &fixture.repo_id,
                "/",
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            fixture.vault.repo_files_browsers_destroy(browser_id);

            recorder.check_recorded(
                |len| assert_eq!(len, 4),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesBrowsersState::default()),
                    1 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loading { loaded: true };
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loaded;
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        RepoFilesBrowsersState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    })
}

#[test]
fn test_reload() {
    with_repo(|fixture| {
        async move {
            let (_, file) = fixture.upload_file("/file.txt", "test").await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.repo_files_browsers.clone(),
            );

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                &fixture.repo_id,
                "/",
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            fixture
                .vault
                .repo_files_browsers_load_files(browser_id)
                .await
                .unwrap();

            fixture.vault.repo_files_browsers_destroy(browser_id);

            recorder.check_recorded(
                |len| assert_eq!(len, 7),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesBrowsersState::default()),
                    1 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loading { loaded: false };
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loading { loaded: false };
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loaded;
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loading { loaded: true };
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    5 => assert_eq!(
                        state,
                        expected_browsers_state(&fixture, &state, |browser| {
                            browser.status = Status::Loaded;
                            browser.file_ids = vec![file.id.clone()];
                        })
                    ),
                    6 => assert_eq!(
                        state,
                        RepoFilesBrowsersState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    })
}

fn expected_browsers_state(
    fixture: &RepoFixture,
    state: &RepoFilesBrowsersState,
    mut patch: impl FnMut(&mut RepoFilesBrowser),
) -> RepoFilesBrowsersState {
    let mut browser = RepoFilesBrowser {
        options: RepoFilesBrowserOptions { select_name: None },
        location: Some(RepoFilesBrowserLocation {
            repo_id: fixture.repo_id.clone(),
            path: "/".into(),
            eventstream_mount_subscription: state
                .browsers
                .get(&1)
                .unwrap()
                .location
                .as_ref()
                .unwrap()
                .eventstream_mount_subscription
                .clone(),
        }),
        status: Status::Initial,
        file_ids: vec![],
        selection: Default::default(),
        sort: Default::default(),
    };

    patch(&mut browser);

    RepoFilesBrowsersState {
        browsers: [(1, browser)].into(),
        next_id: NextId(2),
    }
}
