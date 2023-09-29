use std::collections::HashSet;

use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core::{
    common::state::Status,
    remote::{ApiErrorCode, RemoteError},
    remote_files::state::RemoteFilesLocation,
    repo_create::state::{RepoCreate, RepoCreateForm, RepoCreatesState},
    repos::{
        errors::CreateRepoError,
        state::{RepoConfig, RepoCreated, RepoUnlockMode},
    },
    store,
};
use vault_core_tests::{
    fixtures::user_fixture::UserFixture,
    helpers::{with_repo, with_user},
};
use vault_store::{test_helpers::StateRecorder, NextId};

#[test]
fn test_create() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoCreate],
                |state| state.repo_creates.clone(),
            );

            let (create_id, load_future) = fixture.vault.repo_create_create();
            load_future.await.unwrap();
            fixture
                .vault
                .repo_create_set_password(create_id, "password".into());
            fixture.vault.repo_create_create_repo(create_id).await;
            let repo_id = fixture.vault.with_state(|state| {
                state
                    .repo_creates
                    .creates
                    .get(&1)
                    .unwrap()
                    .created()
                    .unwrap()
                    .repo_id
                    .clone()
            });
            fixture.vault.repo_create_destroy(create_id);

            fixture
                .vault
                .repos_service
                .unlock_repo(&repo_id, "password", RepoUnlockMode::Unlock)
                .await
                .unwrap();
            fixture
                .vault
                .repo_files_service
                .load_files(&repo_id, "/")
                .await
                .unwrap();
            let file_names = fixture.vault.with_state(|state| {
                vault_core::repo_files::selectors::select_files(state, &repo_id, "/")
                    .map(|file| file.decrypted_name().unwrap().to_owned())
                    .collect::<HashSet<_>>()
            });

            assert_eq!(
                file_names,
                HashSet::from([
                    "My private documents".to_owned(),
                    "My private pictures".to_owned(),
                    "My private videos".to_owned(),
                ])
            );

            recorder.check_recorded(
                |len| assert_eq!(len, 7),
                |i, state| match i {
                    0 => assert_eq!(state, RepoCreatesState::default()),
                    1 => assert_eq!(state, expected_create_form_loading(&state)),
                    2 => assert_eq!(state, expected_create_form_loaded(&fixture, &state)),
                    3 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.password = "password".into();
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.password = "password".into();
                            form.create_repo_status = Status::Loading { loaded: false };
                        })
                    ),
                    5 => assert_eq!(state, expected_create_created(&fixture, &state, |_| {})),
                    6 => assert_eq!(
                        state,
                        RepoCreatesState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_create_custom_salt() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoCreate],
                |state| state.repo_creates.clone(),
            );

            let (create_id, load_future) = fixture.vault.repo_create_create();
            load_future.await.unwrap();
            fixture
                .vault
                .repo_create_set_password(create_id, "password".into());
            fixture
                .vault
                .repo_create_set_salt(create_id, Some("salt".into()));
            fixture.vault.repo_create_create_repo(create_id).await;
            fixture.vault.repo_create_destroy(create_id);

            recorder.check_recorded(
                |len| assert_eq!(len, 8),
                |i, state| match i {
                    0 => assert_eq!(state, RepoCreatesState::default()),
                    1 => assert_eq!(state, expected_create_form_loading(&state)),
                    2 => assert_eq!(state, expected_create_form_loaded(&fixture, &state)),
                    3 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.password = "password".into();
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.password = "password".into();
                            form.salt = Some("salt".into());
                        })
                    ),
                    5 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.password = "password".into();
                            form.salt = Some("salt".into());
                            form.create_repo_status = Status::Loading { loaded: false };
                        })
                    ),
                    6 => assert_eq!(state, expected_create_created(&fixture, &state, |_| {})),
                    7 => assert_eq!(
                        state,
                        RepoCreatesState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_create_custom_location() {
    with_user(|fixture| {
        async move {
            fixture.load().await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoCreate],
                |state| state.repo_creates.clone(),
            );

            let (create_id, load_future) = fixture.vault.repo_create_create();
            load_future.await.unwrap();
            fixture.vault.repo_create_set_location(
                create_id,
                RemoteFilesLocation {
                    mount_id: fixture.mount_id.clone(),
                    path: "/custom".into(),
                },
            );
            fixture
                .vault
                .repo_create_set_password(create_id, "password".into());
            fixture.vault.repo_create_create_repo(create_id).await;
            fixture.vault.repo_create_destroy(create_id);

            recorder.check_recorded(
                |len| assert_eq!(len, 8),
                |i, state| match i {
                    0 => assert_eq!(state, RepoCreatesState::default()),
                    1 => assert_eq!(state, expected_create_form_loading(&state)),
                    2 => assert_eq!(state, expected_create_form_loaded(&fixture, &state)),
                    3 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.location = Some(RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            });
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.location = Some(RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            });
                            form.password = "password".into();
                        })
                    ),
                    5 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture, form);
                            form.location = Some(RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            });
                            form.password = "password".into();
                            form.create_repo_status = Status::Loading { loaded: false };
                        })
                    ),
                    6 => assert_eq!(
                        state,
                        expected_create_created(&fixture, &state, |created| {
                            created.config.name = "custom".into();
                            created.config.location = RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            };
                        })
                    ),
                    7 => assert_eq!(
                        state,
                        RepoCreatesState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_create_location_error() {
    with_repo(|fixture| {
        async move {
            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoCreate],
                |state| state.repo_creates.clone(),
            );

            let (create_id, load_future) = fixture.vault.repo_create_create();
            load_future.await.unwrap();
            fixture.vault.repo_create_set_location(
                create_id,
                RemoteFilesLocation {
                    mount_id: fixture.mount_id.clone(),
                    path: "/My safe box".into(),
                },
            );
            fixture
                .vault
                .repo_create_set_password(create_id, "password".into());
            fixture.vault.repo_create_create_repo(create_id).await;
            fixture.vault.repo_create_set_location(
                create_id,
                RemoteFilesLocation {
                    mount_id: fixture.mount_id.clone(),
                    path: "/custom".into(),
                },
            );
            fixture.vault.repo_create_create_repo(create_id).await;
            fixture.vault.repo_create_destroy(create_id);

            recorder.check_recorded(
                |len| assert_eq!(len, 11),
                |i, state| match i {
                    0 => assert_eq!(state, RepoCreatesState::default()),
                    1 => assert_eq!(state, expected_create_form_loading(&state)),
                    2 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                            form.location = None;
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                            form.password = "password".into();
                        })
                    ),
                    5 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                            form.password = "password".into();
                            form.create_repo_status = Status::Loading { loaded: false };
                        })
                    ),
                    6 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                            form.password = "password".into();
                            form.create_repo_status = Status::Error {
                                error: CreateRepoError::RemoteError(RemoteError::ApiError {
                                    code: ApiErrorCode::VaultReposAlreadyExists,
                                    message: "Vault repo already exists for this path.".into(),
                                    request_id: get_create_form_create_error_request_id(&state),
                                    extra: None,
                                    status_code: Some(409),
                                }),
                                loaded: false,
                            };
                        })
                    ),
                    7 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                            form.location = Some(RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            });
                            form.password = "password".into();
                        })
                    ),
                    8 => assert_eq!(
                        state,
                        expected_create_form(&state, |form| {
                            patch_create_form_loaded(&fixture.user_fixture, form);
                            form.location = Some(RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            });
                            form.password = "password".into();
                            form.create_repo_status = Status::Loading { loaded: false };
                        })
                    ),
                    9 => assert_eq!(
                        state,
                        expected_create_created(&fixture.user_fixture, &state, |created| {
                            created.config.name = "custom".into();
                            created.config.location = RemoteFilesLocation {
                                mount_id: fixture.mount_id.clone(),
                                path: "/custom".into(),
                            };
                        })
                    ),
                    10 => assert_eq!(
                        state,
                        RepoCreatesState {
                            next_id: NextId(2),
                            ..Default::default()
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", state),
                },
            );
        }
        .boxed()
    });
}

fn expected_create_form(
    state: &RepoCreatesState,
    mut patch: impl FnMut(&mut RepoCreateForm),
) -> RepoCreatesState {
    let mut form = RepoCreateForm {
        create_load_status: Status::Initial,
        primary_mount_id: None,
        location: None,
        location_dir_picker_id: None,
        password: "".into(),
        salt: Some(
            state
                .creates
                .get(&1)
                .as_ref()
                .unwrap()
                .form()
                .unwrap()
                .salt
                .clone()
                .unwrap_or("expected salt".into()),
        ),
        fill_from_rclone_config_error: None,
        create_repo_status: Status::Initial,
    };

    patch(&mut form);

    RepoCreatesState {
        creates: [(1, RepoCreate::Form(form))].into(),
        next_id: NextId(2),
    }
}

fn expected_create_form_loading(state: &RepoCreatesState) -> RepoCreatesState {
    expected_create_form(&state, |form| {
        form.create_load_status = Status::Loading { loaded: false };
    })
}

fn patch_create_form_loaded(fixture: &UserFixture, form: &mut RepoCreateForm) {
    form.create_load_status = Status::Loaded;
    form.primary_mount_id = Some(fixture.mount_id.clone());
    form.location = Some(RemoteFilesLocation {
        mount_id: fixture.mount_id.clone(),
        path: "/My safe box".into(),
    });
}

fn expected_create_form_loaded(
    fixture: &UserFixture,
    state: &RepoCreatesState,
) -> RepoCreatesState {
    expected_create_form(&state, |form| {
        patch_create_form_loaded(fixture, form);
    })
}

fn get_create_form_create_error_request_id(state: &RepoCreatesState) -> Option<String> {
    match state
        .creates
        .get(&1)
        .as_ref()
        .unwrap()
        .form()
        .unwrap()
        .create_repo_status
        .error()
        .unwrap()
    {
        CreateRepoError::RemoteError(RemoteError::ApiError { request_id, .. }) => {
            request_id.clone()
        }
        _ => None,
    }
}

fn expected_create_created(
    fixture: &UserFixture,
    state: &RepoCreatesState,
    mut patch: impl FnMut(&mut RepoCreated),
) -> RepoCreatesState {
    let state_created = state.creates.get(&1).as_ref().unwrap().created().unwrap();

    let mut created = RepoCreated {
        repo_id: state_created.repo_id.clone(),
        config: RepoConfig {
            name: "My safe box".into(),
            location: RemoteFilesLocation {
                mount_id: fixture.mount_id.clone(),
                path: "/My safe box".into(),
            },
            password: "password".into(),
            salt: state_created.config.salt.clone(),
            rclone_config: state_created.config.rclone_config.clone(),
        },
    };

    patch(&mut created);

    RepoCreatesState {
        creates: [(1, RepoCreate::Created(created))].into(),
        next_id: NextId(2),
    }
}
