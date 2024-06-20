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
    repo_files::errors::LoadFilesError,
    repo_files_details::{
        self,
        state::{
            RepoFilesDetails, RepoFilesDetailsContent, RepoFilesDetailsContentData,
            RepoFilesDetailsContentDataBytes, RepoFilesDetailsContentLoading, RepoFilesDetailsInfo,
            RepoFilesDetailsLocation, RepoFilesDetailsOptions, RepoFilesDetailsState,
        },
    },
    repos,
    repos::errors::{RepoInfoError, RepoLockedError, RepoNotFoundError},
    store,
    transfers::errors::{DownloadableError, TransferError},
    types::{DecryptedName, EncryptedPath, RepoId},
};
use vault_core_tests::{
    fixtures::repo_fixture::RepoFixture,
    helpers::{
        eventstream::eventstream_wait_registered,
        repo_files_details::{details_wait, details_wait_content_loaded},
        transfers::TestDownloadable,
        with_repo, with_user,
    },
};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;
use vault_store::{test_helpers::StateRecorder, NextId};

#[test]
fn test_content() {
    with_repo(|fixture| {
        async move {
            let cipher = fixture.vault.store.with_state(|state| {
                repos::selectors::select_cipher_owned(state, &fixture.repo_id).unwrap()
            });

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
                &fixture.encrypt_path("/file.txt"),
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
                                    bytes: RepoFilesDetailsContentDataBytes::Decrypted(
                                        "test".as_bytes().to_owned(),
                                        cipher.clone(),
                                    ),
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
                &fixture.encrypt_path("/file.txt"),
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
                &fixture.encrypt_path("/file.txt"),
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
                &fixture.encrypt_path("/file.txt"),
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
                    DecryptSizeError::DecryptSizeError(
                        vault_crypto::errors::DecryptSizeError::EncryptedFileTooShort
                    )
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
                                        DecryptSizeError::DecryptSizeError(
                                            vault_crypto::errors::DecryptSizeError::EncryptedFileTooShort
                                        )
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
                &fixture.encrypt_path("/file.txt"),
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

#[test]
fn test_repo_not_loaded() {
    use repo_files_details::selectors::select_info;

    with_user(|user_fixture| {
        async move {
            let fixture = RepoFixture::create(user_fixture).await;

            let fixture1 = fixture.new_session();
            fixture1.user_fixture.login();
            fixture1.user_fixture.load().await;
            fixture1.unlock();
            fixture1.upload_file("/file.txt", "test").await;

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &fixture.encrypt_path("/file.txt"),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![FileCategory::Text],
                        exts: vec![],
                    },
                },
            );

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesDetails],
                |state| state.clone(),
            );

            load_future.await.unwrap();

            fixture.user_fixture.load().await;

            recorder.check_recorded(
                |len| assert_eq!(len, 3),
                |i, state| match i {
                    0 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Loading { loaded: false },
                            file_name: None,
                            file_ext: None,
                            file_category: None,
                            file_modified: None,
                            file_exists: false,
                            content_status: Status::Initial,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: None,
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Initial,
                            is_locked: false,
                        }
                    ),
                    1 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Loading { loaded: false },
                            file_name: None,
                            file_ext: None,
                            file_category: None,
                            file_modified: None,
                            file_exists: false,
                            content_status: Status::Initial,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: None,
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loading { loaded: false },
                            is_locked: false,
                        }
                    ),
                    2 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Error {
                                error: LoadFilesError::RepoLocked(RepoLockedError),
                                loaded: false
                            },
                            file_name: None,
                            file_ext: None,
                            file_category: None,
                            file_modified: None,
                            file_exists: false,
                            content_status: Status::Initial,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: Some("Safe Box is locked".into()),
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loaded,
                            is_locked: true,
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", select_info(&state, details_id)),
                },
            );

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
fn test_repo_locked_unlock() {
    use repo_files_details::selectors::select_info;

    with_user(|user_fixture| {
        async move {
            let fixture = RepoFixture::create(user_fixture).await;
            fixture.user_fixture.load().await;

            let fixture1 = fixture.new_session();
            fixture1.user_fixture.login();
            fixture1.user_fixture.load().await;
            fixture1.unlock();
            fixture1.upload_file("/file.txt", "test").await;

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &fixture.encrypt_path("/file.txt"),
                false,
                RepoFilesDetailsOptions {
                    autosave_interval: Duration::from_secs(20),
                    load_content: FilesFilter {
                        categories: vec![FileCategory::Text],
                        exts: vec![],
                    },
                },
            );

            let recorder = StateRecorder::record(
                fixture.vault.store.clone(),
                &[store::Event::RepoFilesDetails],
                |state| state.clone(),
            );

            load_future.await.unwrap();

            unlock_wait_for_details_loaded(&fixture).await;

            details_wait(fixture.vault.store.clone(), 1, |details| {
                matches!(
                    details.location.as_ref().unwrap().content.status,
                    Status::Loaded
                )
            })
            .await;

            recorder.check_recorded(
                |len| assert_eq!(len, 5),
                |i, state| match i {
                    0 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Error {
                                error: LoadFilesError::RepoLocked(RepoLockedError),
                                loaded: false
                            },
                            file_name: None,
                            file_ext: None,
                            file_category: None,
                            file_modified: None,
                            file_exists: false,
                            content_status: Status::Initial,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: Some("Safe Box is locked".into()),
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loaded,
                            is_locked: true,
                        }
                    ),
                    1 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Loading { loaded: false },
                            file_name: Some(DecryptedName("file.txt".into())),
                            file_ext: Some("txt".into()),
                            file_category: Some(FileCategory::Text),
                            file_modified: None,
                            file_exists: false,
                            content_status: Status::Initial,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: None,
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loaded,
                            is_locked: false,
                        }
                    ),
                    2 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Loaded,
                            file_name: Some(DecryptedName("file.txt".into())),
                            file_ext: Some("txt".into()),
                            file_category: Some(FileCategory::Text),
                            file_modified: Some(
                                select_info(&state, details_id)
                                    .unwrap()
                                    .file_modified
                                    .unwrap()
                            ),
                            file_exists: true,
                            content_status: Status::Initial,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: None,
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loaded,
                            is_locked: false,
                        }
                    ),
                    3 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Loaded,
                            file_name: Some(DecryptedName("file.txt".into())),
                            file_ext: Some("txt".into()),
                            file_category: Some(FileCategory::Text),
                            file_modified: Some(
                                select_info(&state, details_id)
                                    .unwrap()
                                    .file_modified
                                    .unwrap()
                            ),
                            file_exists: true,
                            content_status: Status::Loading { loaded: false },
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: None,
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loaded,
                            is_locked: false,
                        }
                    ),
                    4 => assert_eq!(
                        select_info(&state, details_id).unwrap(),
                        RepoFilesDetailsInfo {
                            repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                            parent_path: Some(EncryptedPath("/".into())),
                            path: Some(&fixture.encrypt_path("/file.txt")),
                            status: Status::Loaded,
                            file_name: Some(DecryptedName("file.txt".into())),
                            file_ext: Some("txt".into()),
                            file_category: Some(FileCategory::Text),
                            file_modified: Some(
                                select_info(&state, details_id)
                                    .unwrap()
                                    .file_modified
                                    .unwrap()
                            ),
                            file_exists: true,
                            content_status: Status::Loaded,
                            transfer_id: None,
                            save_status: Status::Initial,
                            error: None,
                            is_editing: false,
                            is_dirty: false,
                            should_destroy: false,
                            can_save: false,
                            can_download: true,
                            can_copy: true,
                            can_move: true,
                            can_delete: true,
                            repo_status: Status::Loaded,
                            is_locked: false,
                        }
                    ),
                    _ => panic!("unexpected state: {:#?}", select_info(&state, details_id)),
                },
            );

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
fn test_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &fixture.encrypt_path("/file.txt"),
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
                    Status::Loaded
                )
            })
            .await;

            let get_state = || fixture.vault.with_state(|state| state.clone());
            let select_info =
                |state| repo_files_details::selectors::select_info(state, details_id).unwrap();
            fn fix_content_status_loaded<'a>(
                info: RepoFilesDetailsInfo<'a>,
            ) -> RepoFilesDetailsInfo<'a> {
                let mut info = info;
                // fix flaky tests
                if matches!(info.content_status, Status::Loading { loaded: true }) {
                    info.content_status = Status::Loaded;
                };
                info
            }

            let state_before_lock = get_state();
            assert_eq!(
                fix_content_status_loaded(select_info(&state_before_lock)),
                RepoFilesDetailsInfo {
                    repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                    parent_path: Some(EncryptedPath("/".into())),
                    path: Some(&fixture.encrypt_path("/file.txt")),
                    status: Status::Loaded,
                    file_name: Some(DecryptedName("file.txt".into())),
                    file_ext: Some("txt".into()),
                    file_category: Some(FileCategory::Text),
                    file_modified: Some(select_info(&state_before_lock).file_modified.unwrap()),
                    file_exists: true,
                    content_status: Status::Loaded,
                    transfer_id: None,
                    save_status: Status::Initial,
                    error: None,
                    is_editing: false,
                    is_dirty: false,
                    should_destroy: false,
                    can_save: false,
                    can_download: true,
                    can_copy: true,
                    can_move: true,
                    can_delete: true,
                    repo_status: Status::Loaded,
                    is_locked: false,
                }
            );
            assert_eq!(
                state_before_lock
                    .repo_files_details
                    .details
                    .get(&1)
                    .unwrap()
                    .location
                    .as_ref()
                    .unwrap()
                    .content
                    .data
                    .as_ref()
                    .unwrap()
                    .bytes,
                RepoFilesDetailsContentDataBytes::Decrypted(
                    "test".as_bytes().to_owned(),
                    repos::selectors::select_cipher_owned(&state_before_lock, &fixture.repo_id)
                        .unwrap()
                )
            );

            fixture.lock();

            let state_after_lock = get_state();
            assert_eq!(
                fix_content_status_loaded(select_info(&state_after_lock)),
                RepoFilesDetailsInfo {
                    repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                    parent_path: Some(EncryptedPath("/".into())),
                    path: Some(&fixture.encrypt_path("/file.txt")),
                    status: Status::Error {
                        error: LoadFilesError::RepoLocked(RepoLockedError),
                        loaded: true,
                    },
                    file_name: None,
                    file_ext: None,
                    file_category: None,
                    file_modified: Some(select_info(&state_before_lock).file_modified.unwrap()),
                    file_exists: true,
                    content_status: Status::Loaded,
                    transfer_id: None,
                    save_status: Status::Initial,
                    error: Some("Safe Box is locked".into()),
                    is_editing: false,
                    is_dirty: false,
                    should_destroy: false,
                    can_save: false,
                    can_download: true,
                    can_copy: true,
                    can_move: true,
                    can_delete: true,
                    repo_status: Status::Loaded,
                    is_locked: true,
                }
            );
            assert!(matches!(
                state_after_lock
                    .repo_files_details
                    .details
                    .get(&1)
                    .expect("a")
                    .location
                    .as_ref()
                    .expect("b")
                    .content
                    .data
                    .as_ref()
                    .expect("c")
                    .bytes,
                RepoFilesDetailsContentDataBytes::Encrypted(_)
            ));

            unlock_wait_for_details_loaded(&fixture).await;

            let state_after_unlock = get_state();
            assert_eq!(
                select_info(&state_after_unlock),
                select_info(&state_before_lock)
            );
            assert_eq!(
                state_after_unlock
                    .repo_files_details
                    .details
                    .get(&1)
                    .unwrap()
                    .location
                    .as_ref()
                    .unwrap()
                    .content
                    .data
                    .as_ref()
                    .unwrap()
                    .bytes,
                RepoFilesDetailsContentDataBytes::Decrypted(
                    "test".as_bytes().to_owned(),
                    repos::selectors::select_cipher_owned(&state_after_unlock, &fixture.repo_id)
                        .unwrap()
                )
            );

            fixture.remove().await;

            let state_after_remove = get_state();
            assert_eq!(
                fix_content_status_loaded(select_info(&state_after_remove)),
                RepoFilesDetailsInfo {
                    repo_id: Some(&RepoId(fixture.repo_id.0.clone())),
                    parent_path: Some(EncryptedPath("/".into())),
                    path: Some(&fixture.encrypt_path("/file.txt")),
                    status: Status::Error {
                        error: LoadFilesError::RepoNotFound(RepoNotFoundError),
                        loaded: false,
                    },
                    file_name: None,
                    file_ext: None,
                    file_category: None,
                    file_modified: None,
                    file_exists: false,
                    content_status: Status::Loaded,
                    transfer_id: None,
                    save_status: Status::Initial,
                    error: Some("Safe Box not found".into()),
                    is_editing: false,
                    is_dirty: false,
                    should_destroy: false,
                    can_save: false,
                    can_download: true,
                    can_copy: true,
                    can_move: true,
                    can_delete: true,
                    repo_status: Status::Error {
                        error: RepoInfoError::RepoNotFound(RepoNotFoundError),
                        loaded: true
                    },
                    is_locked: false,
                }
            );
            assert_eq!(
                state_after_remove
                    .repo_files_details
                    .details
                    .get(&1)
                    .unwrap()
                    .location
                    .as_ref()
                    .unwrap()
                    .content
                    .data
                    .as_ref()
                    .unwrap()
                    .bytes,
                RepoFilesDetailsContentDataBytes::Decrypted(
                    "test".as_bytes().to_owned(),
                    repos::selectors::select_cipher_owned(&state_after_unlock, &fixture.repo_id)
                        .unwrap()
                )
            );

            fixture
                .vault
                .repo_files_details_destroy(details_id)
                .await
                .unwrap();
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
        id: 1,
        options: RepoFilesDetailsOptions {
            autosave_interval: Duration::from_secs(20),
            load_content: FilesFilter {
                categories: vec![],
                exts: vec![],
            },
        },
        location: Some(RepoFilesDetailsLocation {
            repo_id: fixture.repo_id.clone(),
            path: fixture.encrypt_path("/file.txt"),
            name: fixture.encrypt_filename("file.txt"),
            decrypted_name: Some(Ok(DecryptedName("file.txt".into()))),
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
        repo_status: Status::Loaded,
        is_locked: false,
        repo_files_subscription_id: state.details.get(&1).unwrap().repo_files_subscription_id,
    };

    patch(&mut details);

    RepoFilesDetailsState {
        details: [(1, details)].into(),
        next_id: NextId(2),
    }
}

async fn unlock_wait_for_details_loaded(fixture: &RepoFixture) {
    // wait for loading and loaded, otherwise we have flaky tests
    let loading_store = fixture.vault.store.clone();
    let loaded_store = fixture.vault.store.clone();
    let loaded_future = store::wait_for(
        fixture.vault.store.clone(),
        &[store::Event::RepoFilesDetails],
        move |_| {
            loading_store.with_state(|state| {
                state
                    .repo_files_details
                    .details
                    .get(&1)
                    .filter(|details| matches!(details.status, Status::Loading { .. }))
                    .map(|_| ())
            })
        },
    )
    .then(|_| {
        store::wait_for(
            fixture.vault.store.clone(),
            &[store::Event::RepoFilesDetails],
            move |_| {
                loaded_store.with_state(|state| {
                    state
                        .repo_files_details
                        .details
                        .get(&1)
                        .filter(|details| matches!(details.status, Status::Loaded))
                        .map(|_| ())
                })
            },
        )
    });

    fixture.unlock();

    loaded_future.await;
}

#[test]
fn test_eventstream() {
    with_repo(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let fixture1 = fixture.new_session();
            fixture1.user_fixture.login();
            fixture1.user_fixture.load().await;
            fixture1.unlock();

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &fixture.encrypt_path("/file.txt"),
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
                    .filter(|x| match &x.bytes {
                        RepoFilesDetailsContentDataBytes::Encrypted(_) => false,
                        RepoFilesDetailsContentDataBytes::Decrypted(bytes, _) => {
                            bytes.as_slice() == "test1".as_bytes()
                        }
                    })
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

            fixture1.upload_file("/file.txt", "test").await;

            let (details_id, load_future) = fixture.vault.repo_files_details_create(
                fixture.repo_id.clone(),
                &fixture.encrypt_path("/file.txt"),
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

            vault_load_future.await.unwrap();

            eventstream_wait_registered(
                fixture.vault.store.clone(),
                &fixture.mount_id,
                &fixture.path,
            )
            .await;

            fixture.unlock();

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
                    .filter(|x| match &x.bytes {
                        RepoFilesDetailsContentDataBytes::Encrypted(_) => false,
                        RepoFilesDetailsContentDataBytes::Decrypted(bytes, _) => {
                            bytes.as_slice() == "test1".as_bytes()
                        }
                    })
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
