use futures::{join, FutureExt};
use similar_asserts::assert_eq;
use vault_core::{
    common::state::Status,
    dialogs,
    repo_files::{
        errors::LoadFilesError,
        state::{RepoFilesBreadcrumb, RepoFilesSort, RepoFilesSortField},
    },
    repo_files_browsers::{
        self,
        state::{
            RepoFilesBrowser, RepoFilesBrowserInfo, RepoFilesBrowserItem, RepoFilesBrowserLocation,
            RepoFilesBrowserOptions, RepoFilesBrowsersState,
        },
    },
    repos::errors::{RepoInfoError, RepoLockedError, RepoNotFoundError},
    selection::state::SelectionSummary,
    sort::state::SortDirection,
    store,
    types::{EncryptedName, EncryptedPath, RepoFileId},
};
use vault_core_tests::{
    fixtures::repo_fixture::RepoFixture,
    helpers::{eventstream::eventstream_wait_registered, with_repo, with_user},
};
use vault_store::{test_helpers::StateRecorder, NextId};

#[test]
fn test_repo_not_loaded() {
    use repo_files_browsers::selectors::select_info;

    with_user(|user_fixture| {
        async move {
            let fixture = RepoFixture::create(user_fixture).await;

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.clone(),
            );

            load_future.await.unwrap();

            fixture.user_fixture.load().await;

            recorder.check_recorded(
                |len| assert_eq!(len, 3),
                |i, state| match i {
                    0 => assert_eq!(
                        select_info(&state, browser_id).unwrap(),
                        RepoFilesBrowserInfo {
                            repo_id: Some(&fixture.repo_id),
                            path: Some(&EncryptedPath("/".into())),
                            selection_summary: SelectionSummary::None,
                            sort: RepoFilesSort {
                                field: RepoFilesSortField::Name,
                                direction: SortDirection::Asc
                            },
                            status: Status::Loading { loaded: false },
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
                            items: vec![],
                            breadcrumbs: None,
                            repo_status: Status::Initial,
                            is_locked: false,
                        }
                    ),
                    1 => assert_eq!(
                        select_info(&state, browser_id).unwrap(),
                        RepoFilesBrowserInfo {
                            repo_id: Some(&fixture.repo_id),
                            path: Some(&EncryptedPath("/".into())),
                            selection_summary: SelectionSummary::None,
                            sort: RepoFilesSort {
                                field: RepoFilesSortField::Name,
                                direction: SortDirection::Asc
                            },
                            status: Status::Loading { loaded: false },
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
                            items: vec![],
                            breadcrumbs: None,
                            repo_status: Status::Loading { loaded: false },
                            is_locked: false,
                        }
                    ),
                    2 => assert_eq!(
                        select_info(&state, browser_id).unwrap(),
                        RepoFilesBrowserInfo {
                            repo_id: Some(&fixture.repo_id),
                            path: Some(&EncryptedPath("/".into())),
                            selection_summary: SelectionSummary::None,
                            sort: RepoFilesSort {
                                field: RepoFilesSortField::Name,
                                direction: SortDirection::Asc
                            },
                            status: Status::Error {
                                error: LoadFilesError::RepoLocked(RepoLockedError),
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
                            items: vec![],
                            breadcrumbs: None,
                            repo_status: Status::Loaded,
                            is_locked: true,
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", select_info(&state, browser_id)),
                },
            );

            fixture.vault.repo_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_repo_locked_unlock() {
    use repo_files_browsers::selectors::select_info;

    with_user(|user_fixture| {
        async move {
            let fixture = RepoFixture::create(user_fixture).await;
            fixture.user_fixture.load().await;

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.clone(),
            );

            unlock_wait_for_browser_loaded(&fixture).await;

            recorder.check_recorded(
                |len| assert_eq!(len, 3),
                |i, state| match i {
                    0 => assert_eq!(
                        select_info(&state, browser_id).unwrap(),
                        RepoFilesBrowserInfo {
                            repo_id: Some(&fixture.repo_id),
                            path: Some(&EncryptedPath("/".into())),
                            selection_summary: SelectionSummary::None,
                            sort: RepoFilesSort {
                                field: RepoFilesSortField::Name,
                                direction: SortDirection::Asc
                            },
                            status: Status::Error {
                                error: LoadFilesError::RepoLocked(RepoLockedError),
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
                            items: vec![],
                            breadcrumbs: None,
                            repo_status: Status::Loaded,
                            is_locked: true,
                        }
                    ),
                    1 => assert_eq!(
                        select_info(&state, browser_id).unwrap(),
                        RepoFilesBrowserInfo {
                            repo_id: Some(&fixture.repo_id),
                            path: Some(&EncryptedPath("/".into())),
                            selection_summary: SelectionSummary::None,
                            sort: RepoFilesSort {
                                field: RepoFilesSortField::Name,
                                direction: SortDirection::Asc
                            },
                            status: Status::Loading { loaded: false },
                            title: Some("My safe box".into()),
                            total_count: 0,
                            total_size: 0,
                            selected_count: 0,
                            selected_size: 0,
                            selected_file: None,
                            can_download_selected: false,
                            can_copy_selected: false,
                            can_move_selected: false,
                            can_delete_selected: false,
                            items: vec![],
                            breadcrumbs: Some(&[RepoFilesBreadcrumb {
                                id: fixture.get_file_id("/"),
                                repo_id: fixture.repo_id.clone(),
                                path: EncryptedPath("/".into()),
                                name: "My safe box".into(),
                                last: true
                            }]),
                            repo_status: Status::Loaded,
                            is_locked: false,
                        }
                    ),
                    2 => assert_eq!(
                        select_info(&state, browser_id).unwrap(),
                        RepoFilesBrowserInfo {
                            repo_id: Some(&fixture.repo_id),
                            path: Some(&EncryptedPath("/".into())),
                            selection_summary: SelectionSummary::None,
                            sort: RepoFilesSort {
                                field: RepoFilesSortField::Name,
                                direction: SortDirection::Asc
                            },
                            status: Status::Loaded,
                            title: Some("My safe box".into()),
                            total_count: 0,
                            total_size: 0,
                            selected_count: 0,
                            selected_size: 0,
                            selected_file: None,
                            can_download_selected: false,
                            can_copy_selected: false,
                            can_move_selected: false,
                            can_delete_selected: false,
                            items: vec![],
                            breadcrumbs: Some(&[RepoFilesBreadcrumb {
                                id: fixture.get_file_id("/"),
                                repo_id: fixture.repo_id.clone(),
                                path: EncryptedPath("/".into()),
                                name: "My safe box".into(),
                                last: true
                            }]),
                            repo_status: Status::Loaded,
                            is_locked: false,
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", select_info(&state, browser_id)),
                },
            );

            fixture.vault.repo_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let get_state = || fixture.vault.with_state(|state| state.clone());
            let select_info =
                |state| repo_files_browsers::selectors::select_info(state, browser_id).unwrap();
            let select_items =
                |state| repo_files_browsers::selectors::select_items(state, browser_id);

            let (_, file) = fixture.upload_file("/file.txt", "test").await;
            let dir = fixture.create_dir("/dir").await;

            let state_before_lock = get_state();
            assert_eq!(
                select_info(&state_before_lock),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some(&EncryptedPath("/".into())),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: Status::Loaded,
                    title: Some("My safe box".into()),
                    total_count: 2,
                    total_size: 4,
                    selected_count: 0,
                    selected_size: 0,
                    selected_file: None,
                    can_download_selected: false,
                    can_copy_selected: false,
                    can_move_selected: false,
                    can_delete_selected: false,
                    items: vec![
                        RepoFilesBrowserItem {
                            file: &dir,
                            is_selected: false,
                        },
                        RepoFilesBrowserItem {
                            file: &file,
                            is_selected: false,
                        }
                    ],
                    breadcrumbs: Some(&[RepoFilesBreadcrumb {
                        id: fixture.get_file_id("/"),
                        repo_id: fixture.repo_id.clone(),
                        path: EncryptedPath("/".into()),
                        name: "My safe box".into(),
                        last: true
                    }]),
                    repo_status: Status::Loaded,
                    is_locked: false,
                }
            );

            fixture.lock();

            let state_after_lock = get_state();
            assert_eq!(
                select_info(&state_after_lock),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some(&EncryptedPath("/".into())),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: Status::Error {
                        error: LoadFilesError::RepoLocked(RepoLockedError),
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
                    items: vec![],
                    breadcrumbs: None,
                    repo_status: Status::Loaded,
                    is_locked: true,
                }
            );

            unlock_wait_for_browser_loaded(&fixture).await;

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
                    path: Some(&EncryptedPath("/".into())),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: Status::Error {
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
                    items: vec![],
                    breadcrumbs: None,
                    repo_status: Status::Error {
                        error: RepoInfoError::RepoNotFound(RepoNotFoundError),
                        loaded: true
                    },
                    is_locked: false,
                }
            );

            fixture.vault.repo_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_repo_decrypt_path_error() {
    with_repo(|fixture| {
        async move {
            fixture
                .create_dir_encrypted(&EncryptedPath("/".into()), EncryptedName("dir".into()))
                .await;
            let (_, file) = fixture
                .upload_file_encrypted(
                    &EncryptedPath("/dir".into()),
                    fixture.encrypt_filename("file.txt".into()),
                    "test",
                )
                .await;

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/dir".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let get_state = || fixture.vault.with_state(|state| state.clone());
            let select_info =
                |state| repo_files_browsers::selectors::select_info(state, browser_id).unwrap();

            let state = get_state();
            assert_eq!(
                select_info(&state),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some(&EncryptedPath("/dir".into())),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: Status::Loaded,
                    title: Some("dir".into()),
                    total_count: 1,
                    total_size: 4,
                    selected_count: 0,
                    selected_size: 0,
                    selected_file: None,
                    can_download_selected: false,
                    can_copy_selected: false,
                    can_move_selected: false,
                    can_delete_selected: false,
                    items: vec![RepoFilesBrowserItem {
                        file: &file,
                        is_selected: false,
                    }],
                    breadcrumbs: Some(&[
                        RepoFilesBreadcrumb {
                            id: fixture.get_file_id("/"),
                            repo_id: fixture.repo_id.clone(),
                            path: EncryptedPath("/".into()),
                            name: "My safe box".into(),
                            last: false
                        },
                        RepoFilesBreadcrumb {
                            id: RepoFileId(format!("{}:/dir", fixture.repo_id.0)),
                            repo_id: fixture.repo_id.clone(),
                            path: EncryptedPath("/dir".into()),
                            name: "dir".into(),
                            last: true
                        }
                    ]),
                    repo_status: Status::Loaded,
                    is_locked: false,
                }
            );

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
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
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
                .load_files(&fixture.repo_id, &EncryptedPath("/".into()))
                .await
                .unwrap();

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.repo_files_browsers.clone(),
            );

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
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
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
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
        id: 1,
        options: RepoFilesBrowserOptions { select_name: None },
        location: Some(RepoFilesBrowserLocation {
            repo_id: fixture.repo_id.clone(),
            path: EncryptedPath("/".into()),
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
        breadcrumbs: Some(vec![RepoFilesBreadcrumb {
            id: fixture.get_file_id("/"),
            repo_id: fixture.repo_id.clone(),
            path: EncryptedPath("/".into()),
            name: "My safe box".into(),
            last: true,
        }]),
        file_ids: vec![],
        selection: Default::default(),
        sort: Default::default(),
        repo_status: Status::Loaded,
        is_locked: false,
    };

    patch(&mut browser);

    RepoFilesBrowsersState {
        browsers: [(1, browser)].into(),
        next_id: NextId(2),
        last_sort: Default::default(),
    }
}

async fn unlock_wait_for_browser_loaded(fixture: &RepoFixture) {
    // wait for loading and loaded, otherwise we have flaky tests
    let loading_store = fixture.vault.store.clone();
    let loaded_store = fixture.vault.store.clone();
    let loaded_future = store::wait_for(
        fixture.vault.store.clone(),
        &[store::Event::RepoFilesBrowsers],
        move |_| {
            loading_store.with_state(|state| {
                state
                    .repo_files_browsers
                    .browsers
                    .get(&1)
                    .filter(|browser| matches!(browser.status, Status::Loading { .. }))
                    .map(|_| ())
            })
        },
    )
    .then(|_| {
        store::wait_for(
            fixture.vault.store.clone(),
            &[store::Event::RepoFilesBrowsers],
            move |_| {
                loaded_store.with_state(|state| {
                    state
                        .repo_files_browsers
                        .browsers
                        .get(&1)
                        .filter(|browser| matches!(browser.status, Status::Loaded))
                        .map(|_| ())
                })
            },
        )
    });

    fixture.unlock();

    loaded_future.await;
}

#[test]
fn test_create_dir() {
    with_repo(|fixture| {
        async move {
            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let create_dir_future = fixture.vault.repo_files_browsers_create_dir(browser_id);

            let dialog_vault = fixture.vault.clone();
            let dialog_future = fixture.fake_remote.tokio_runtime.spawn(async move {
                let wait_store = dialog_vault.store.clone();
                let dialog_id =
                    store::wait_for(wait_store.clone(), &[store::Event::Dialogs], move |_| {
                        wait_store.with_state(|state| {
                            dialogs::selectors::select_dialogs(state)
                                .iter()
                                .next()
                                .map(|dialog| dialog.id.clone())
                        })
                    })
                    .await;

                dialog_vault.dialogs_set_input_value(dialog_id, "dir".into());

                dialog_vault.dialogs_confirm(dialog_id);
            });

            let (create_dir_res, _) = join!(create_dir_future, dialog_future);
            let (name, path) = create_dir_res.unwrap();

            assert_eq!(name.0, "dir");
            assert_eq!(path, fixture.encrypt_path("/dir"));

            fixture.vault.repo_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_create_dir_validation() {
    with_repo(|fixture| {
        async move {
            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let create_dir_future = fixture.vault.repo_files_browsers_create_dir(browser_id);

            let dialog_vault = fixture.vault.clone();
            let dialog_future = fixture.fake_remote.tokio_runtime.spawn(async move {
                let wait_store = dialog_vault.store.clone();
                let dialog_id =
                    store::wait_for(wait_store.clone(), &[store::Event::Dialogs], move |_| {
                        wait_store.with_state(|state| {
                            dialogs::selectors::select_dialogs(state)
                                .iter()
                                .next()
                                .map(|dialog| dialog.id.clone())
                        })
                    })
                    .await;

                dialog_vault.dialogs_set_input_value(dialog_id, "/".into());

                assert!(!dialog_vault.store.with_state(|state| {
                    dialogs::selectors::select_dialog(state, dialog_id)
                        .unwrap()
                        .is_input_value_valid
                }));

                dialog_vault.dialogs_confirm(dialog_id);

                assert!(dialog_vault.store.with_state(|state| {
                    dialogs::selectors::select_dialog(state, dialog_id).is_none()
                }));
            });

            let (create_dir_res, _) = join!(create_dir_future, dialog_future);

            assert_eq!(
                create_dir_res.unwrap_err().to_string(),
                "Invalid name or path"
            );

            fixture.vault.repo_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_eventstream() {
    with_repo(|fixture| {
        async move {
            let fixture1 = fixture.new_session();
            fixture1.user_fixture.login();
            fixture1.user_fixture.load().await;
            fixture1.unlock();

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();
            eventstream_wait_registered(
                fixture.vault.store.clone(),
                &fixture.mount_id,
                &fixture.path,
            )
            .await;

            fixture1.upload_file("/file.txt", "test").await;

            let wait_for_store = fixture.vault.store.clone();
            store::wait_for(
                wait_for_store.clone(),
                &[store::Event::RepoFilesBrowsers],
                move |_| {
                    wait_for_store.with_state(|state| {
                        repo_files_browsers::selectors::select_info(state, browser_id)
                            .filter(|info| {
                                info.items
                                    .iter()
                                    .find(|item| item.file.name_lower_force() == "file.txt")
                                    .is_some()
                            })
                            .map(|_| ())
                    })
                },
            )
            .await;

            fixture.vault.remote_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}

#[test]
fn test_eventstream_not_loaded() {
    with_user(|fixture| {
        async move {
            let fixture = RepoFixture::create(fixture).await;
            let vault_load_future = fixture.vault.load().unwrap();

            let fixture1 = fixture.new_session();
            fixture1.user_fixture.login();
            fixture1.user_fixture.load().await;
            fixture1.unlock();

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &EncryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            vault_load_future.await.unwrap();

            eventstream_wait_registered(
                fixture.vault.store.clone(),
                &fixture.mount_id,
                &fixture.path,
            )
            .await;

            fixture.unlock();

            // wait until loaded after unlock to prevent flaky tests
            let wait_for_store = fixture.vault.store.clone();
            store::wait_for(
                wait_for_store.clone(),
                &[store::Event::RepoFilesBrowsers],
                move |_| {
                    wait_for_store.with_state(|state| {
                        repo_files_browsers::selectors::select_info(state, browser_id)
                            .filter(|info| matches!(info.status, Status::Loaded))
                            .map(|_| ())
                    })
                },
            )
            .await;

            fixture1.upload_file("/file.txt", "test").await;

            let wait_for_store = fixture.vault.store.clone();
            store::wait_for(
                wait_for_store.clone(),
                &[store::Event::RepoFilesBrowsers],
                move |_| {
                    wait_for_store.with_state(|state| {
                        repo_files_browsers::selectors::select_info(state, browser_id)
                            .filter(|info| {
                                info.items
                                    .iter()
                                    .find(|item| item.file.name_lower_force() == "file.txt")
                                    .is_some()
                            })
                            .map(|_| ())
                    })
                },
            )
            .await;

            fixture.vault.remote_files_browsers_destroy(browser_id);
        }
        .boxed()
    });
}
