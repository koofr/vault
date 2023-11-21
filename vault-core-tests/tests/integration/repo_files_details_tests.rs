use std::{
    future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{http::StatusCode, response::IntoResponse};
use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_core::{
    cipher::errors::DecryptSizeError,
    common::state::Status,
    files::{file_category::FileCategory, files_filter::FilesFilter},
    remote_files,
    repo_files_details::state::{
        RepoFilesDetails, RepoFilesDetailsContent, RepoFilesDetailsContentData,
        RepoFilesDetailsContentLoading, RepoFilesDetailsLocation, RepoFilesDetailsOptions,
        RepoFilesDetailsState,
    },
    store,
    transfers::errors::{DownloadableError, TransferError},
    types::{DecryptedName, DecryptedPath},
};
use vault_core_tests::{
    fixtures::repo_fixture::RepoFixture,
    helpers::{
        eventstream::eventstream_wait_registered,
        repo_files_details::{details_wait, details_wait_content_loaded},
        transfers::TestDownloadable,
        with_repo,
    },
};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;
use vault_store::{test_helpers::StateRecorder, NextId};

#[test]
fn test_content() {
    with_repo(|fixture| {
        async move {
            let (upload_result, _) = fixture.upload_file("/file.txt", "test").await;

            // remove file from state so that it is loaded before the content is loaded to prevent flaky tests
            fixture
                .vault
                .store
                .mutate(|state, notify, mutation_state, mutation_notify| {
                    remote_files::mutations::file_removed(
                        state,
                        notify,
                        mutation_state,
                        mutation_notify,
                        &upload_result.remote_file.mount_id,
                        &upload_result.remote_file.path,
                    );
                });

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesDetails],
                |state| state.repo_files_details.clone(),
            );

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/file.txt".into()),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![FileCategory::Text],
                        exts: vec![],
                    },
                },
            );
            load_future.await.unwrap();

            details_wait_content_loaded(fixture.vault.store.clone(), 1).await;

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();

            recorder.check_recorded(
                |len| assert_eq!(len, 6),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesDetailsState::default()),
                    1 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details
                                .options
                                .load_content
                                .categories
                                .push(FileCategory::Text);
                            details.status = Status::Loading { loaded: false };
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details
                                .options
                                .load_content
                                .categories
                                .push(FileCategory::Text);
                            details.status = Status::Loaded;
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details
                                .options
                                .load_content
                                .categories
                                .push(FileCategory::Text);
                            details.status = Status::Loaded;

                            if let Some(location) = details.location.as_mut() {
                                location.content.status = Status::Loading { loaded: false };
                                location.content.loading = Some(RepoFilesDetailsContentLoading {
                                    remote_size: upload_result.remote_file.size,
                                    remote_modified: upload_result.remote_file.modified,
                                    remote_hash: upload_result.remote_file.hash.clone(),
                                });
                            }
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details
                                .options
                                .load_content
                                .categories
                                .push(FileCategory::Text);
                            details.status = Status::Loaded;

                            if let Some(location) = details.location.as_mut() {
                                location.content.status = Status::Loaded;
                                location.content.data = Some(RepoFilesDetailsContentData {
                                    bytes: "test".as_bytes().to_owned(),
                                    remote_size: upload_result.remote_file.size,
                                    remote_modified: upload_result.remote_file.modified,
                                    remote_hash: upload_result.remote_file.hash.clone(),
                                });
                                location.content.version = 1;
                            }
                        })
                    ),
                    5 => assert_eq!(
                        state,
                        RepoFilesDetailsState {
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
fn test_content_loaded_error() {
    with_repo(|fixture| {
        async move {
            let download_counter = Arc::new(AtomicUsize::new(0));
            let interceptor_download_counter = download_counter.clone();

            fixture.fake_remote.intercept(Box::new(move |parts| {
                if parts.uri.path().contains("/content/api")
                    && parts.uri.path().contains("/files/get")
                {
                    interceptor_download_counter.fetch_add(1, Ordering::SeqCst);
                    InterceptorResult::Response(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                } else {
                    InterceptorResult::Ignore
                }
            }));

            fixture.upload_file("/file.txt", "test").await;

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/file.txt".into()),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![FileCategory::Text],
                        exts: vec![],
                    },
                },
            );
            load_future.await.unwrap();

            details_wait(fixture.vault.store.clone(), 1, |details| {
                matches!(
                    details.location.as_ref().unwrap().content.status,
                    Status::Error { .. }
                )
            })
            .await;

            // one retry on server errors in http client
            assert_eq!(download_counter.load(Ordering::SeqCst), 2);

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();
        }
        .boxed()
    });
}

#[test]
fn test_download() {
    with_repo(|fixture| {
        async move {
            let (upload_result, _) = fixture.upload_file("/file.txt", "test").await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesDetails],
                |state| state.repo_files_details.clone(),
            );

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/file.txt".into()),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![],
                        exts: vec![],
                    },
                },
            );
            load_future.await.unwrap();

            let (downloadable, content_future) = TestDownloadable::string();

            fixture
                .vault
                .repo_files_details_download(details_id, Box::new(downloadable))
                .await
                .unwrap();

            assert_eq!(content_future.await.unwrap(), "test");

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();

            recorder.check_recorded(
                |len| assert_eq!(len, 6),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesDetailsState::default()),
                    1 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loading { loaded: true };
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;

                            if let Some(location) = details.location.as_mut() {
                                location.content.status = Status::Loading { loaded: false };
                                location.content.loading = Some(RepoFilesDetailsContentLoading {
                                    remote_size: upload_result.remote_file.size,
                                    remote_modified: upload_result.remote_file.modified,
                                    remote_hash: upload_result.remote_file.hash.clone(),
                                });
                                location.content.transfer_id = Some(1);
                            }
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;

                            if let Some(location) = details.location.as_mut() {
                                location.content.status = Status::Loaded;
                                location.content.version = 1;
                            }
                        })
                    ),
                    5 => assert_eq!(
                        state,
                        RepoFilesDetailsState {
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
fn test_download_size_decryption_error() {
    with_repo(|fixture| {
        async move {
            let encrypted_path = format!(
                "/My safe box/{}",
                fixture
                    .vault
                    .repo_files_service
                    .encrypt_filename(&fixture.repo_id, &DecryptedName("file.txt".into()))
                    .unwrap()
                    .0
            );

            fixture
                .user_fixture
                .upload_remote_file(&encrypted_path, "test")
                .await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesDetails],
                |state| state.repo_files_details.clone(),
            );

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/file.txt".into()),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![],
                        exts: vec![],
                    },
                },
            );
            load_future.await.unwrap();

            let (downloadable, content_future) = TestDownloadable::string();

            let res = fixture
                .vault
                .repo_files_details_download(details_id, Box::new(downloadable))
                .await;
            assert_eq!(
                res,
                Err(TransferError::DecryptSizeError(
                    DecryptSizeError::EncryptedFileTooShort
                ))
            );

            assert!(content_future.await.is_none());

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();

            recorder.check_recorded(
                |len| assert_eq!(len, 5),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesDetailsState::default()),
                    1 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loading { loaded: true };
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;

                            if let Some(location) = details.location.as_mut() {
                                location.content.status = Status::Error {
                                    error: TransferError::DecryptSizeError(
                                        DecryptSizeError::EncryptedFileTooShort,
                                    ),
                                    loaded: false,
                                };
                            }
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        RepoFilesDetailsState {
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
fn test_download_downloadable_error() {
    with_repo(|fixture| {
        async move {
            let _ = fixture.upload_file("/file.txt", "test").await;

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesDetails],
                |state| state.repo_files_details.clone(),
            );

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/file.txt".into()),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![],
                        exts: vec![],
                    },
                },
            );
            load_future.await.unwrap();

            let (mut downloadable, content_future) = TestDownloadable::string();
            downloadable.exists_fn = Box::new(|_, _| {
                future::ready(Err(DownloadableError::LocalFileError(
                    "downloadable exists error".into(),
                )))
                .boxed()
            });

            let res = fixture
                .vault
                .repo_files_details_download(details_id, Box::new(downloadable))
                .await;
            assert_eq!(
                res,
                Err(TransferError::LocalFileError(
                    "downloadable exists error".into()
                ))
            );

            assert!(content_future.await.is_none());

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();

            recorder.check_recorded(
                |len| assert_eq!(len, 5),
                |i, state| match i {
                    0 => assert_eq!(state, RepoFilesDetailsState::default()),
                    1 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loading { loaded: true };
                        })
                    ),
                    2 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;
                        })
                    ),
                    3 => assert_eq!(
                        state,
                        expected_details_state(&fixture, &state, |details| {
                            details.status = Status::Loaded;

                            if let Some(location) = details.location.as_mut() {
                                location.content.status = Status::Error {
                                    error: TransferError::LocalFileError(
                                        "downloadable exists error".into(),
                                    ),
                                    loaded: false,
                                };
                            }
                        })
                    ),
                    4 => assert_eq!(
                        state,
                        RepoFilesDetailsState {
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

fn expected_details_state(
    fixture: &RepoFixture,
    state: &RepoFilesDetailsState,
    mut patch: impl FnMut(&mut RepoFilesDetails),
) -> RepoFilesDetailsState {
    let mut details = RepoFilesDetails {
        options: RepoFilesDetailsOptions {
            autosave_interval: Duration::from_secs(20),
            load_content: FilesFilter {
                categories: vec![],
                exts: vec![],
            },
        },
        location: Some(RepoFilesDetailsLocation {
            repo_id: fixture.repo_id.clone(),
            path: DecryptedPath("/file.txt".into()),
            eventstream_mount_subscription: state
                .details
                .get(&1)
                .unwrap()
                .location
                .as_ref()
                .unwrap()
                .eventstream_mount_subscription
                .clone(),
            content: RepoFilesDetailsContent {
                status: Status::Initial,
                data: None,
                loading: None,
                version: 0,
                transfer_id: None,
            },
            is_editing: false,
            is_dirty: false,
            save_status: Status::Initial,
            delete_status: Status::Initial,
            should_destroy: false,
        }),
        status: Status::Initial,
        repo_files_subscription_id: state.details.get(&1).unwrap().repo_files_subscription_id,
    };

    patch(&mut details);

    RepoFilesDetailsState {
        details: [(1, details)].into(),
        next_id: NextId(2),
    }
}

#[test]
fn test_eventstream() {
    with_repo(|fixture| {
        async move {
            let (_, file) = fixture.upload_file("/file.txt", "test").await;

            let fixture1 = fixture.new_session();
            fixture1.user_fixture.login();
            fixture1.user_fixture.load().await;
            fixture1.unlock().await;

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &DecryptedPath("/file.txt".into()),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![FileCategory::Text],
                        exts: vec![],
                    },
                },
            );
            load_future.await.unwrap();
            eventstream_wait_registered(
                fixture.vault.store.clone(),
                &fixture.mount_id,
                &fixture.path,
            )
            .await;

            details_wait(fixture.vault.store.clone(), 1, |details| {
                matches!(
                    details.location.as_ref().unwrap().content.status,
                    Status::Loaded
                )
            })
            .await;

            fixture1.upload_file("/file.txt", "test1").await;

            details_wait(fixture.vault.store.clone(), 1, |details| {
                matches!(
                    details.location.as_ref().unwrap().content.status,
                    Status::Loaded
                ) && details
                    .location
                    .as_ref()
                    .unwrap()
                    .content
                    .data
                    .as_ref()
                    .map(|x| String::from_utf8(x.bytes.clone()) == Ok("test1".to_string()))
                    .is_some()
            })
            .await;

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();
        }
        .boxed()
    });
}
