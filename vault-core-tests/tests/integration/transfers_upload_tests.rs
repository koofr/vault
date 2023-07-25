use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{http::StatusCode, response::IntoResponse};
use futures::{future, io::Cursor, FutureExt};
use similar_asserts::assert_eq;

use vault_core::{
    common::state::{BoxAsyncRead, SizeInfo},
    files::file_category::FileCategory,
    store::{self, test_helpers::StoreWatcher, NextId},
    transfers::{
        errors::{TransferError, UploadableError},
        state::{Transfer, TransferState, TransferType, TransfersState, UploadTransfer},
    },
};
use vault_core_tests::helpers::transfers::{
    capture_upload_uri, check_recorded, patch_transfer, transfer_abort_when, transfer_do_when,
    transfers_recorder, uploaded_server_error, with_transfers, TestUploadable,
};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;

#[test]
fn test_upload() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_uri_receiver = capture_upload_uri(&fixture.fake_remote);

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        assert!(upload_uri_receiver
            .await
            .unwrap()
            .query()
            .unwrap()
            .contains("&size=52"));

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_name_path() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "path/to/file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        let patch = |t: &mut Transfer| {
            t.typ = TransferType::Upload(UploadTransfer {
                repo_id: repo_id.clone(),
                parent_path: "/path/to".into(),
                name_rel_path: Some("path/to".into()),
                original_name: "file.txt".into(),
                name: "file.txt".into(),
            });
            t.name = "path/to/file.txt".into();
        };

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, patch)
                ),
                2 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_processing(&repo_id, &transfers, 1),
                        1,
                        patch
                    )
                ),
                3 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring(&repo_id, &transfers, 1),
                        1,
                        patch
                    )
                ),
                4 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring_progress(&repo_id, &transfers, 1),
                        1,
                        patch
                    )
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_name_path_autorename() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        fixture.create_dir("/path").await;
        fixture.create_dir("/path/to").await;
        fixture.upload_file("/path/to/file.txt", "old").await;

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "path/to/file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file (1).txt");

        let patch = |t: &mut Transfer| {
            t.typ = TransferType::Upload(UploadTransfer {
                repo_id: repo_id.clone(),
                parent_path: "/path/to".into(),
                name_rel_path: Some("path/to".into()),
                original_name: "file.txt".into(),
                name: "file.txt".into(),
            });
            t.name = "path/to/file.txt".into();
        };

        let patch_processed = |t: &mut Transfer| {
            t.typ = TransferType::Upload(UploadTransfer {
                repo_id: repo_id.clone(),
                parent_path: "/path/to".into(),
                name_rel_path: Some("path/to".into()),
                original_name: "file.txt".into(),
                name: "file (1).txt".into(),
            });
            t.name = "path/to/file (1).txt".into();
        };

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, patch)
                ),
                2 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_processing(&repo_id, &transfers, 1),
                        1,
                        patch
                    )
                ),
                3 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring(&repo_id, &transfers, 1),
                        1,
                        patch_processed
                    )
                ),
                4 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring_progress(&repo_id, &transfers, 1),
                        1,
                        patch_processed
                    )
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_size_estimate() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_uri_receiver = capture_upload_uri(&fixture.fake_remote);

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Estimate(4))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    future::ready(Ok((
                        Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                        SizeInfo::Estimate(4),
                    )))
                    .boxed()
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        assert!(!upload_uri_receiver
            .await
            .unwrap()
            .query()
            .unwrap()
            .contains("&size="));

        let patch = |t: &mut Transfer| {
            t.size = SizeInfo::Estimate(4);
        };

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, patch)
                ),
                2 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_processing(&repo_id, &transfers, 1),
                        1,
                        patch
                    ),
                ),
                3 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring(&repo_id, &transfers, 1),
                        1,
                        patch
                    ),
                ),
                4 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring_progress(&repo_id, &transfers, 1),
                        1,
                        patch
                    ),
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_size_unknown() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_uri_receiver = capture_upload_uri(&fixture.fake_remote);

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Unknown)).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    future::ready(Ok((
                        Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                        SizeInfo::Unknown,
                    )))
                    .boxed()
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        assert!(!upload_uri_receiver
            .await
            .unwrap()
            .query()
            .unwrap()
            .contains("&size="));

        let patch = |t: &mut Transfer| {
            t.size = SizeInfo::Unknown;
        };

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, patch)
                    }
                ),
                2 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(
                            expected_transfers_processing(&repo_id, &transfers, 1),
                            1,
                            patch
                        )
                    }
                ),
                3 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(
                            expected_transfers_transferring(&repo_id, &transfers, 1),
                            1,
                            patch
                        )
                    }
                ),
                4 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(
                            expected_transfers_transferring_progress(&repo_id, &transfers, 1),
                            1,
                            patch
                        )
                    }
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_size_unknown_to_estimate() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_uri_receiver = capture_upload_uri(&fixture.fake_remote);

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Unknown)).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    future::ready(Ok((
                        Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                        SizeInfo::Estimate(4),
                    )))
                    .boxed()
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        assert!(!upload_uri_receiver
            .await
            .unwrap()
            .query()
            .unwrap()
            .contains("&size="));

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, |t| {
                            t.size = SizeInfo::Unknown
                        })
                    }
                ),
                2 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(
                            expected_transfers_processing(&repo_id, &transfers, 1),
                            1,
                            |t| t.size = SizeInfo::Unknown
                        )
                    }
                ),
                3 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring(&repo_id, &transfers, 1),
                        1,
                        |t| t.size = SizeInfo::Estimate(4)
                    )
                ),
                4 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring_progress(&repo_id, &transfers, 1),
                        1,
                        |t| t.size = SizeInfo::Estimate(4)
                    )
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_size_unknown_to_exact() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_uri_receiver = capture_upload_uri(&fixture.fake_remote);

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Unknown)).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    future::ready(Ok((
                        Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                        SizeInfo::Exact(4),
                    )))
                    .boxed()
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        assert!(upload_uri_receiver
            .await
            .unwrap()
            .query()
            .unwrap()
            .contains("&size=52"));

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, |t| {
                            t.size = SizeInfo::Unknown
                        })
                    }
                ),
                2 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 0,
                        ..patch_transfer(
                            expected_transfers_processing(&repo_id, &transfers, 1),
                            1,
                            |t| t.size = SizeInfo::Unknown
                        )
                    }
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_size_estimate_to_exact() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_uri_receiver = capture_upload_uri(&fixture.fake_remote);

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Estimate(5))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    future::ready(Ok((
                        Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                        SizeInfo::Exact(4),
                    )))
                    .boxed()
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        assert!(upload_uri_receiver
            .await
            .unwrap()
            .query()
            .unwrap()
            .contains("&size=52"));

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 5,
                        ..patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, |t| {
                            t.size = SizeInfo::Estimate(5)
                        })
                    }
                ),
                2 => assert_eq!(
                    transfers,
                    TransfersState {
                        total_bytes: 5,
                        ..patch_transfer(
                            expected_transfers_processing(&repo_id, &transfers, 1),
                            1,
                            |t| t.size = SizeInfo::Estimate(5)
                        )
                    }
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                ),
                5 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_concurrency() {
    with_transfers(|fixture| async move {
        let (_, create_future_1) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file1.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(4))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || future::pending().boxed()),
            }),
        );
        let future_1 = create_future_1.await.unwrap();

        let (_, create_future_2) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file2.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(5))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || future::pending().boxed()),
            }),
        );
        let future_2 = create_future_2.await.unwrap();

        let (transfer_id_3, create_future_3) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file3.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(5))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || future::pending().boxed()),
            }),
        );
        let future_3 = create_future_3.await.unwrap();

        let (transfer_id_4, create_future_4) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file4.txt".into(),
            TestUploadable::string("test"),
        );
        let future_4 = create_future_4.await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(fixture.vault.store.with_state(|state| matches!(
            state
                .transfers
                .transfers
                .get(&transfer_id_4)
                .as_ref()
                .unwrap()
                .state,
            TransferState::Waiting
        )));

        fixture.vault.transfers_abort(transfer_id_3);

        assert!(matches!(future_3.await, Err(TransferError::Aborted)));

        assert!(matches!(future_4.await, Ok(_)));

        fixture.vault.transfers_abort_all();

        assert!(matches!(future_1.await, Err(TransferError::Aborted)));

        assert!(matches!(future_2.await, Err(TransferError::Aborted)));

        assert_eq!(
            fixture
                .vault
                .store
                .with_state(|state| state.transfers.total_count),
            0
        );
    });
}

#[test]
fn test_upload_load_root_error() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let bundle_counter = Arc::new(AtomicUsize::new(0));
        let interceptor_bundle_counter = bundle_counter.clone();

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/bundle") {
                // one retry on server errors in http client
                if interceptor_bundle_counter.fetch_add(1, Ordering::SeqCst) < 2 {
                    InterceptorResult::Response(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                } else {
                    InterceptorResult::Ignore
                }
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "path/to/file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        let patch = |t: &mut Transfer| {
            t.typ = TransferType::Upload(UploadTransfer {
                repo_id: repo_id.clone(),
                parent_path: "/path/to".into(),
                name_rel_path: Some("path/to".into()),
                original_name: "file.txt".into(),
                name: "file.txt".into(),
            });
            t.name = "path/to/file.txt".into();
        };

        check_recorded(
            recorder,
            |len| assert_eq!(len, 8),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    patch_transfer(expected_transfers_waiting(&repo_id, &transfers), 1, patch)
                ),
                2 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_processing(&repo_id, &transfers, 1),
                        1,
                        patch
                    ),
                ),
                3 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_waiting_failed(&repo_id, &transfers, 1),
                        1,
                        patch
                    ),
                ),
                4 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_processing(&repo_id, &transfers, 2),
                        1,
                        patch
                    ),
                ),
                5 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring(&repo_id, &transfers, 2),
                        1,
                        patch
                    ),
                ),
                6 => assert_eq!(
                    transfers,
                    patch_transfer(
                        expected_transfers_transferring_progress(&repo_id, &transfers, 2),
                        1,
                        patch
                    ),
                ),
                7 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_size_error() {
    with_transfers(|fixture| async move {
        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(|| {
                    future::ready(Err(UploadableError::LocalFileError("size error".into()))).boxed()
                }),
                is_retriable_fn: Box::new(|| panic!("unreachable")),
                reader_fn: Box::new(|| panic!("unreachable")),
            }),
        );

        assert!(
            matches!(create_future.await, Err(TransferError::LocalFileError(err)) if err == "size error")
        );

        check_recorded(
            recorder,
            |len| assert_eq!(len, 1),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_is_retriable_error() {
    with_transfers(|fixture| async move {
        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(|| future::ready(Ok(SizeInfo::Exact(4))).boxed()),
                is_retriable_fn: Box::new(|| {
                    future::ready(Err(UploadableError::LocalFileError(
                        "is retriable error".into(),
                    )))
                    .boxed()
                }),
                reader_fn: Box::new(|| panic!("unreachable")),
            }),
        );

        assert!(
            matches!(create_future.await, Err(TransferError::LocalFileError(err)) if err == "is retriable error")
        );

        check_recorded(
            recorder,
            |len| assert_eq!(len, 1),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_reader_error_retriable() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let recorder = transfers_recorder(&fixture.vault);

        let reader_counter = Arc::new(AtomicUsize::new(0));

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(|| future::ready(Ok(SizeInfo::Exact(4))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    if reader_counter.fetch_add(1, Ordering::SeqCst) == 0 {
                        future::ready(Err(UploadableError::LocalFileError("reader error".into())))
                            .boxed()
                    } else {
                        future::ready(Ok((
                            Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                            SizeInfo::Exact(4),
                        )))
                        .boxed()
                    }
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        check_recorded(
            recorder,
            |len| assert_eq!(len, 8),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 2)
                ),
                5 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 2)
                ),
                6 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 2)
                ),
                7 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_abort_immediately() {
    with_transfers(|fixture| async move {
        let recorder = transfers_recorder(&fixture.vault);

        let (transfer_id, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        fixture.vault.transfers_abort(transfer_id);

        assert!(matches!(create_future.await, Err(TransferError::Aborted)));

        check_recorded(
            recorder,
            |len| assert_eq!(len, 1),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_abort_waiting() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Waiting)
        });

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        assert!(matches!(future.await, Err(TransferError::Aborted)));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 3),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_abort_processing() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Processing)
        });

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        assert!(matches!(future.await, Err(TransferError::Aborted)));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 4),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_abort_transferring() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Transferring)
        });

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        assert!(matches!(future.await, Err(TransferError::Aborted)));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 5),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_abort_all() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let recorder = transfers_recorder(&fixture.vault);

        let watcher_vault = fixture.vault.clone();
        let watcher = StoreWatcher::watch_store(
            fixture.vault.store.clone(),
            &[store::Event::Transfers],
            move |store, _| {
                if store.with_state(|state| {
                    state
                        .transfers
                        .transfers
                        .get(&1)
                        .filter(|t| matches!(t.state, TransferState::Processing))
                        .is_some()
                        && state
                            .transfers
                            .transfers
                            .get(&2)
                            .filter(|t| matches!(t.state, TransferState::Processing))
                            .is_some()
                }) {
                    watcher_vault.transfers_abort_all();
                }
            },
        );

        let (_, create_future_1) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file1.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(4))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || future::pending().boxed()),
            }),
        );
        let future_1 = create_future_1.await.unwrap();

        let (_, create_future_2) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file2.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(5))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || future::pending().boxed()),
            }),
        );
        let future_2 = create_future_2.await.unwrap();

        assert!(matches!(future_1.await, Err(TransferError::Aborted)));
        assert!(matches!(future_2.await, Err(TransferError::Aborted)));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 6),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(
                    transfers,
                    TransfersState {
                        transfers: [(
                            1,
                            Transfer {
                                id: 1,
                                typ: TransferType::Upload(UploadTransfer {
                                    repo_id: repo_id.to_owned(),
                                    parent_path: "/".into(),
                                    name_rel_path: None,
                                    original_name: "file1.txt".into(),
                                    name: "file1.txt".into(),
                                }),
                                name: "file1.txt".into(),
                                size: SizeInfo::Exact(4),
                                category: FileCategory::Text,
                                started: None,
                                is_persistent: false,
                                is_retriable: true,
                                state: TransferState::Waiting,
                                transferred_bytes: 0,
                                attempts: 0,
                                order: 0,
                            },
                        )]
                        .into(),
                        next_id: NextId(2),
                        started: None,
                        last_progress_update: transfers.last_progress_update,
                        transferring_count: 0,
                        transferring_uploads_count: 0,
                        transferring_downloads_count: 0,
                        done_count: 0,
                        failed_count: 0,
                        retriable_count: 0,
                        total_count: 1,
                        done_bytes: 0,
                        failed_bytes: 0,
                        total_bytes: 4,
                    }
                ),
                2 => assert_eq!(
                    transfers,
                    TransfersState {
                        transfers: [(
                            1,
                            Transfer {
                                id: 1,
                                typ: TransferType::Upload(UploadTransfer {
                                    repo_id: repo_id.to_owned(),
                                    parent_path: "/".into(),
                                    name_rel_path: None,
                                    original_name: "file1.txt".into(),
                                    name: "file1.txt".into(),
                                }),
                                name: "file1.txt".into(),
                                size: SizeInfo::Exact(4),
                                category: FileCategory::Text,
                                started: Some(
                                    transfers
                                        .transfers
                                        .get(&1)
                                        .and_then(|t| t.started)
                                        .unwrap_or(9999),
                                ),
                                is_persistent: false,
                                is_retriable: true,
                                state: TransferState::Processing,
                                transferred_bytes: 0,
                                attempts: 1,
                                order: 0,
                            },
                        )]
                        .into(),
                        next_id: NextId(2),
                        started: Some(transfers.started.unwrap_or(999)),
                        last_progress_update: transfers.last_progress_update,
                        transferring_count: 1,
                        transferring_uploads_count: 1,
                        transferring_downloads_count: 0,
                        done_count: 0,
                        failed_count: 0,
                        retriable_count: 0,
                        total_count: 1,
                        done_bytes: 0,
                        failed_bytes: 0,
                        total_bytes: 4,
                    }
                ),
                3 => assert_eq!(
                    transfers,
                    TransfersState {
                        transfers: [
                            (
                                1,
                                Transfer {
                                    id: 1,
                                    typ: TransferType::Upload(UploadTransfer {
                                        repo_id: repo_id.to_owned(),
                                        parent_path: "/".into(),
                                        name_rel_path: None,
                                        original_name: "file1.txt".into(),
                                        name: "file1.txt".into(),
                                    }),
                                    name: "file1.txt".into(),
                                    size: SizeInfo::Exact(4),
                                    category: FileCategory::Text,
                                    started: Some(
                                        transfers
                                            .transfers
                                            .get(&1)
                                            .and_then(|t| t.started)
                                            .unwrap_or(9999),
                                    ),
                                    is_persistent: false,
                                    is_retriable: true,
                                    state: TransferState::Processing,
                                    transferred_bytes: 0,
                                    attempts: 1,
                                    order: 0,
                                },
                            ),
                            (
                                2,
                                Transfer {
                                    id: 2,
                                    typ: TransferType::Upload(UploadTransfer {
                                        repo_id: repo_id.to_owned(),
                                        parent_path: "/".into(),
                                        name_rel_path: None,
                                        original_name: "file2.txt".into(),
                                        name: "file2.txt".into(),
                                    }),
                                    name: "file2.txt".into(),
                                    size: SizeInfo::Exact(5),
                                    category: FileCategory::Text,
                                    started: None,
                                    is_persistent: false,
                                    is_retriable: true,
                                    state: TransferState::Waiting,
                                    transferred_bytes: 0,
                                    attempts: 0,
                                    order: 1,
                                },
                            )
                        ]
                        .into(),
                        next_id: NextId(3),
                        started: Some(transfers.started.unwrap_or(999)),
                        last_progress_update: transfers.last_progress_update,
                        transferring_count: 1,
                        transferring_uploads_count: 1,
                        transferring_downloads_count: 0,
                        done_count: 0,
                        failed_count: 0,
                        retriable_count: 0,
                        total_count: 2,
                        done_bytes: 0,
                        failed_bytes: 0,
                        total_bytes: 9,
                    }
                ),
                4 => assert_eq!(
                    transfers,
                    TransfersState {
                        transfers: [
                            (
                                1,
                                Transfer {
                                    id: 1,
                                    typ: TransferType::Upload(UploadTransfer {
                                        repo_id: repo_id.to_owned(),
                                        parent_path: "/".into(),
                                        name_rel_path: None,
                                        original_name: "file1.txt".into(),
                                        name: "file1.txt".into(),
                                    }),
                                    name: "file1.txt".into(),
                                    size: SizeInfo::Exact(4),
                                    category: FileCategory::Text,
                                    started: Some(
                                        transfers
                                            .transfers
                                            .get(&1)
                                            .and_then(|t| t.started)
                                            .unwrap_or(9999),
                                    ),
                                    is_persistent: false,
                                    is_retriable: true,
                                    state: TransferState::Processing,
                                    transferred_bytes: 0,
                                    attempts: 1,
                                    order: 0,
                                },
                            ),
                            (
                                2,
                                Transfer {
                                    id: 2,
                                    typ: TransferType::Upload(UploadTransfer {
                                        repo_id: repo_id.to_owned(),
                                        parent_path: "/".into(),
                                        name_rel_path: None,
                                        original_name: "file2.txt".into(),
                                        name: "file2.txt".into(),
                                    }),
                                    name: "file2.txt".into(),
                                    size: SizeInfo::Exact(5),
                                    category: FileCategory::Text,
                                    started: Some(
                                        transfers
                                            .transfers
                                            .get(&2)
                                            .and_then(|t| t.started)
                                            .unwrap_or(9999),
                                    ),
                                    is_persistent: false,
                                    is_retriable: true,
                                    state: TransferState::Processing,
                                    transferred_bytes: 0,
                                    attempts: 1,
                                    order: 1,
                                },
                            )
                        ]
                        .into(),
                        next_id: NextId(3),
                        started: Some(transfers.started.unwrap_or(999)),
                        last_progress_update: transfers.last_progress_update,
                        transferring_count: 2,
                        transferring_uploads_count: 2,
                        transferring_downloads_count: 0,
                        done_count: 0,
                        failed_count: 0,
                        retriable_count: 0,
                        total_count: 2,
                        done_bytes: 0,
                        failed_bytes: 0,
                        total_bytes: 9,
                    }
                ),
                5 => assert_eq!(
                    transfers,
                    TransfersState {
                        next_id: NextId(3),
                        ..expected_tranfers_done()
                    }
                ),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_fail_autoretry_succeed() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_counter = Arc::new(AtomicUsize::new(0));
        let interceptor_upload_counter = upload_counter.clone();
        let interceptor_store = fixture.vault.store.clone();

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/put")
            {
                if interceptor_upload_counter.fetch_add(1, Ordering::SeqCst) == 0 {
                    uploaded_server_error(interceptor_store.clone(), 1, 4)
                } else {
                    InterceptorResult::Ignore
                }
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        check_recorded(
            recorder,
            |len| assert_eq!(len, 9),
            |i, transfers| {
                match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                    2 => assert_eq!(
                        transfers,
                        expected_transfers_processing(&repo_id, &transfers, 1)
                    ),
                    3 => assert_eq!(
                        transfers,
                        expected_transfers_transferring(&repo_id, &transfers, 1)
                    ),
                    4 => assert_eq!(
                        transfers,
                        expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                    ),
                    5 => assert_eq!(
                        transfers,
                        expected_transfers_waiting_failed(&repo_id, &transfers, 1)
                    ),
                    6 => assert_eq!(
                        transfers,
                        expected_transfers_processing(&repo_id, &transfers, 2)
                    ),
                    7 => assert_eq!(
                        transfers,
                        expected_transfers_transferring(&repo_id, &transfers, 2)
                    ),
                    // no progress because last_progress_update is set from the first attempt
                    8 => assert_eq!(transfers, expected_tranfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                }
            },
        );
    });
}

#[test]
fn test_upload_fail_autoretry_fail() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let interceptor_store = fixture.vault.store.clone();

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/put")
            {
                uploaded_server_error(interceptor_store.clone(), 1, 4)
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Failed { .. })
        });

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await;
        // TODO should this be Aborted or the last error from the Failed transfer
        assert!(matches!(res, Err(TransferError::Aborted)));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 19),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                ),
                5 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 1)
                ),
                6 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 2)
                ),
                7 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 2)
                ),
                8 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 2)
                ),
                9 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 3)
                ),
                10 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 3)
                ),
                11 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 3)
                ),
                12 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 4)
                ),
                13 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 4)
                ),
                14 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 4)
                ),
                15 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 5)
                ),
                16 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 5)
                ),
                17 => assert_eq!(
                    transfers,
                    expected_transfers_failed(&repo_id, &transfers, 5)
                ),
                18 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_fail_autoretry_retry() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let upload_counter = Arc::new(AtomicUsize::new(0));
        let interceptor_upload_counter = upload_counter.clone();
        let interceptor_store = fixture.vault.store.clone();

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/put")
            {
                if interceptor_upload_counter.fetch_add(1, Ordering::SeqCst) < 5 {
                    uploaded_server_error(interceptor_store.clone(), 1, 4)
                } else {
                    InterceptorResult::Ignore
                }
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_do_when(
            fixture.vault.clone(),
            1,
            |t| matches!(t.state, TransferState::Failed { .. }),
            |vault| vault.transfers_retry(1),
        );

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            TestUploadable::string("test"),
        );
        let future = create_future.await.unwrap();

        let res = future.await.unwrap();
        assert_eq!(res.name, "file.txt");

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 22),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                ),
                5 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 1)
                ),
                6 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 2)
                ),
                7 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 2)
                ),
                8 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 2)
                ),
                9 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 3)
                ),
                10 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 3)
                ),
                11 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 3)
                ),
                12 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 4)
                ),
                13 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 4)
                ),
                14 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 4)
                ),
                15 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 5)
                ),
                16 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 5)
                ),
                17 => assert_eq!(
                    transfers,
                    expected_transfers_failed(&repo_id, &transfers, 5)
                ),
                18 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 5)
                ),
                19 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 6)
                ),
                20 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 6)
                ),
                21 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_fail_autoretry_not_retriable() {
    with_transfers(|fixture| async move {
        let repo_id = fixture.repo_id.clone();

        let interceptor_store = fixture.vault.store.clone();

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/put")
            {
                uploaded_server_error(interceptor_store.clone(), 1, 4)
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Failed { .. })
        });

        let reader_counter = Arc::new(AtomicUsize::new(0));

        let (_, create_future) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(|| future::ready(Ok(SizeInfo::Exact(4))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    if reader_counter.fetch_add(1, Ordering::SeqCst) == 0 {
                        future::ready(Ok((
                            Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                            SizeInfo::Exact(4),
                        )))
                        .boxed()
                    } else {
                        future::ready(Err(UploadableError::NotRetriable)).boxed()
                    }
                }),
            }),
        );
        let future = create_future.await.unwrap();

        let res = future.await;
        // TODO should this be Aborted or the last error from the Failed transfer
        assert!(matches!(res, Err(TransferError::Aborted)));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 9),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&repo_id, &transfers)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 1)
                ),
                3 => assert_eq!(
                    transfers,
                    expected_transfers_transferring(&repo_id, &transfers, 1)
                ),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&repo_id, &transfers, 1)
                ),
                5 => assert_eq!(
                    transfers,
                    expected_transfers_waiting_failed(&repo_id, &transfers, 1)
                ),
                6 => assert_eq!(
                    transfers,
                    expected_transfers_processing(&repo_id, &transfers, 2)
                ),
                7 => assert_eq!(
                    transfers,
                    patch_transfer(expected_transfers_failed(&repo_id, &transfers, 2), 1, |t| {
                        t.is_retriable = false
                    })
                ),
                8 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_upload_retry_all() {
    with_transfers(|fixture| async move {
        let fail = Arc::new(AtomicBool::new(true));

        let watcher_vault = fixture.vault.clone();
        let watcher_fail = fail.clone();
        let watcher = StoreWatcher::watch_store(
            fixture.vault.store.clone(),
            &[store::Event::Transfers],
            move |store, _| {
                if store.with_state(|state| {
                    state
                        .transfers
                        .transfers
                        .get(&1)
                        .filter(|t| matches!(t.state, TransferState::Failed { .. }))
                        .is_some()
                        && state
                            .transfers
                            .transfers
                            .get(&2)
                            .filter(|t| matches!(t.state, TransferState::Failed { .. }))
                            .is_some()
                }) {
                    watcher_fail.store(false, Ordering::SeqCst);

                    watcher_vault.transfers_retry_all();
                }
            },
        );

        let fail_1 = fail.clone();
        let (_, create_future_1) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file1.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(4))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    if fail_1.load(Ordering::SeqCst) {
                        future::ready(Err(UploadableError::LocalFileError("reader error".into())))
                            .boxed()
                    } else {
                        future::ready(Ok((
                            Box::pin(Cursor::new("test".as_bytes().to_vec())) as BoxAsyncRead,
                            SizeInfo::Exact(4),
                        )))
                        .boxed()
                    }
                }),
            }),
        );
        let future_1 = create_future_1.await.unwrap();

        let fail_2 = fail.clone();
        let (_, create_future_2) = fixture.vault.transfers_upload(
            fixture.repo_id.clone(),
            "/".into(),
            "file2.txt".into(),
            Box::new(TestUploadable {
                size_fn: Box::new(move || future::ready(Ok(SizeInfo::Exact(5))).boxed()),
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                reader_fn: Box::new(move || {
                    if fail_2.load(Ordering::SeqCst) {
                        future::ready(Err(UploadableError::LocalFileError("reader error".into())))
                            .boxed()
                    } else {
                        future::ready(Ok((
                            Box::pin(Cursor::new("test2".as_bytes().to_vec())) as BoxAsyncRead,
                            SizeInfo::Exact(4),
                        )))
                        .boxed()
                    }
                }),
            }),
        );
        let future_2 = create_future_2.await.unwrap();

        let res_1 = future_1.await.unwrap();
        assert_eq!(res_1.name, "file1.txt");

        let res_2 = future_2.await.unwrap();
        assert_eq!(res_2.name, "file2.txt");

        drop(watcher);
    });
}

fn expected_transfers_waiting(repo_id: &str, transfers: &TransfersState) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo_id.to_owned(),
                    parent_path: "/".into(),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                state: TransferState::Waiting,
                transferred_bytes: 0,
                attempts: 0,
                order: 0,
            },
        )]
        .into(),
        next_id: NextId(2),
        started: None,
        last_progress_update: transfers.last_progress_update,
        transferring_count: 0,
        transferring_uploads_count: 0,
        transferring_downloads_count: 0,
        done_count: 0,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 0,
        failed_bytes: 0,
        total_bytes: 4,
    }
}

fn expected_transfers_processing(
    repo_id: &str,
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo_id.to_owned(),
                    parent_path: "/".into(),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: Some(
                    transfers
                        .transfers
                        .get(&1)
                        .and_then(|t| t.started)
                        .unwrap_or(9999),
                ),
                is_persistent: false,
                is_retriable: true,
                state: TransferState::Processing,
                transferred_bytes: 0,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: NextId(2),
        started: Some(transfers.started.unwrap_or(999)),
        last_progress_update: transfers.last_progress_update,
        transferring_count: 1,
        transferring_uploads_count: 1,
        transferring_downloads_count: 0,
        done_count: 0,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 0,
        failed_bytes: 0,
        total_bytes: 4,
    }
}

fn expected_transfers_transferring(
    repo_id: &str,
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo_id.to_owned(),
                    parent_path: "/".into(),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: Some(
                    transfers
                        .transfers
                        .get(&1)
                        .and_then(|t| t.started)
                        .unwrap_or(9999),
                ),
                is_persistent: false,
                is_retriable: true,
                state: TransferState::Transferring,
                transferred_bytes: 0,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: NextId(2),
        started: Some(transfers.started.unwrap_or(999)),
        last_progress_update: transfers.last_progress_update,
        transferring_count: 1,
        transferring_uploads_count: 1,
        transferring_downloads_count: 0,
        done_count: 0,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 0,
        failed_bytes: 0,
        total_bytes: 4,
    }
}

fn expected_transfers_transferring_progress(
    repo_id: &str,
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo_id.to_owned(),
                    parent_path: "/".into(),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: Some(
                    transfers
                        .transfers
                        .get(&1)
                        .and_then(|t| t.started)
                        .unwrap_or(9999),
                ),
                is_persistent: false,
                is_retriable: true,
                state: TransferState::Transferring,
                transferred_bytes: 4,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: NextId(2),
        started: Some(transfers.started.unwrap_or(999)),
        last_progress_update: transfers.last_progress_update,
        transferring_count: 1,
        transferring_uploads_count: 1,
        transferring_downloads_count: 0,
        done_count: 0,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 4,
        failed_bytes: 0,
        total_bytes: 4,
    }
}

fn expected_transfers_waiting_failed(
    repo_id: &str,
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo_id.to_owned(),
                    parent_path: "/".into(),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                state: TransferState::Waiting,
                transferred_bytes: 0,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: NextId(2),
        started: Some(transfers.started.unwrap_or(999)),
        last_progress_update: transfers.last_progress_update,
        transferring_count: 0,
        transferring_uploads_count: 0,
        transferring_downloads_count: 0,
        done_count: 0,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 0,
        failed_bytes: 0,
        total_bytes: 4,
    }
}

fn expected_transfers_failed(
    repo_id: &str,
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo_id.to_owned(),
                    parent_path: "/".into(),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                state: match &transfers.transfers.get(&1).unwrap().state {
                    TransferState::Failed { error } => TransferState::Failed {
                        error: error.clone(),
                    },
                    state => panic!("Expected transfer state to be Failed, got {:?}", state),
                },
                transferred_bytes: 0,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: NextId(2),
        started: None,
        last_progress_update: transfers.last_progress_update,
        transferring_count: 0,
        transferring_uploads_count: 0,
        transferring_downloads_count: 0,
        done_count: 0,
        failed_count: 1,
        retriable_count: 1,
        total_count: 1,
        done_bytes: 0,
        failed_bytes: 4,
        total_bytes: 4,
    }
}

fn expected_tranfers_done() -> TransfersState {
    TransfersState {
        next_id: NextId(2),
        ..Default::default()
    }
}
