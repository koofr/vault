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
    repos::errors::{RepoLockedError, RepoNotFoundError},
    selection::state::SelectionSummary,
    sort::state::SortDirection,
    store,
    types::{DecryptedName, DecryptedPath},
};
use vault_core_tests::{
    fixtures::repo_fixture::RepoFixture,
    helpers::{eventstream::eventstream_wait_registered, with_repo},
};
use vault_store::{test_helpers::StateRecorder, NextId};

#[test]
fn test_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/".into()),
                RepoFilesBrowserOptions { select_name: None },
            );
            load_future.await.unwrap();

            let get_state = || fixture.vault.with_state(|state| state.clone());
            let select_info = |state| {
                vault_core::repo_files_browsers::selectors::select_info(state, browser_id).unwrap()
            };
            let select_items =
                |state| vault_core::repo_files_browsers::selectors::select_items(state, browser_id);

            let root_id = vault_core::repo_files::selectors::get_file_id(
                &fixture.repo_id,
                &DecryptedPath("/".into()),
            );
            let (_, file) = fixture.upload_file("/file.txt", "test").await;
            let dir = fixture.create_dir("/dir").await;

            let state_before_lock = get_state();
            assert_eq!(
                select_info(&state_before_lock),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some(&DecryptedPath("/".into())),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: vault_core::common::state::Status::Loaded,
                    title: Some(DecryptedName("My safe box".to_owned())),
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
                    breadcrumbs: vec![RepoFilesBreadcrumb {
                        id: root_id.clone(),
                        repo_id: fixture.repo_id.clone(),
                        path: DecryptedPath("/".into()),
                        name: DecryptedName("My safe box".into()),
                        last: true
                    }],
                }
            );

            fixture.lock();

            let state_after_lock = get_state();
            assert_eq!(
                select_info(&state_after_lock),
                RepoFilesBrowserInfo {
                    repo_id: Some(&fixture.repo_id),
                    path: Some(&DecryptedPath("/".into())),
                    selection_summary: SelectionSummary::None,
                    sort: RepoFilesSort {
                        field: RepoFilesSortField::Name,
                        direction: SortDirection::Asc
                    },
                    status: vault_core::common::state::Status::Error {
                        error: LoadFilesError::RepoLocked(RepoLockedError),
                        loaded: false
                    },
                    title: Some(DecryptedName("My safe box".to_owned())),
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
                    breadcrumbs: vec![RepoFilesBreadcrumb {
                        id: root_id.clone(),
                        repo_id: fixture.repo_id.clone(),
                        path: DecryptedPath("/".into()),
                        name: DecryptedName("My safe box".into()),
                        last: true
                    }],
                }
            );

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
                    path: Some(&DecryptedPath("/".into())),
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
                    items: vec![],
                    breadcrumbs: vec![]
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
                &DecryptedPath("/".into()),
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
                .load_files(&fixture.repo_id, &DecryptedPath("/".into()))
                .await
                .unwrap();

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesBrowsers],
                |state| state.repo_files_browsers.clone(),
            );

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/".into()),
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
                &DecryptedPath("/".into()),
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
            path: DecryptedPath("/".into()),
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
        last_sort: Default::default(),
    }
}

#[test]
fn test_create_dir() {
    with_repo(|fixture| {
        async move {
            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/".into()),
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
            assert_eq!(path.0, "/dir");

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
                &DecryptedPath("/".into()),
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
            fixture1.unlock().await;

            let (browser_id, load_future) = fixture.vault.repo_files_browsers_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/".into()),
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
