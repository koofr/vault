use std::time::Duration;

use futures::AsyncReadExt;
use similar_asserts::assert_eq;

use vault_core::{
    common::state::SizeInfo,
    file_types::file_category::FileCategory,
    transfers::state::{Transfer, TransferState, TransferType, TransfersState},
};
use vault_core_tests::helpers::transfers::{
    check_recorded, transfer_abort_when, transfers_recorder, with_transfers,
};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;

#[test]
fn test_download_reader() {
    with_transfers(|fixture| async move {
        fixture.upload_file("/file.txt", "test").await;

        let recorder = transfers_recorder(&fixture.vault);

        let reader = fixture
            .vault
            .repo_files_get_file_reader(&fixture.repo_id, "/file.txt")
            .unwrap()
            .reader()
            .await
            .unwrap();

        let (_, mut reader) = fixture.vault.transfers_download_reader(reader);

        let mut content = String::new();

        reader.reader.read_to_string(&mut content).await.unwrap();

        assert_eq!(content, "test");

        check_recorded(
            recorder,
            |len| assert_eq!(len, 4),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                2 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&transfers, 1)
                ),
                3 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_download_reader_fail() {
    with_transfers(|fixture| async move {
        fixture.upload_file("/file.txt", "test").await;

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/get")
            {
                InterceptorResult::delayed_abort_response_body(Duration::from_millis(50))
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let reader = fixture
            .vault
            .repo_files_get_file_reader(&fixture.repo_id, "/file.txt")
            .unwrap()
            .reader()
            .await
            .unwrap();

        let (_, mut reader) = fixture.vault.transfers_download_reader(reader);

        let mut content = String::new();
        let res = reader.reader.read_to_string(&mut content).await;
        assert!(matches!(res, Err(std::io::Error { .. })));

        check_recorded(
            recorder,
            |len| assert_eq!(len, 3),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                2 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

#[test]
fn test_download_reader_abort() {
    with_transfers(|fixture| async move {
        fixture.upload_file("/file.txt", "test").await;

        fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/get")
            {
                InterceptorResult::delayed_response_body(Duration::from_millis(50))
            } else {
                InterceptorResult::Ignore
            }
        }));

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 0, |t| {
            matches!(t.state, TransferState::Transferring)
        });

        let reader = fixture
            .vault
            .repo_files_get_file_reader(&fixture.repo_id, "/file.txt")
            .unwrap()
            .reader()
            .await
            .unwrap();

        let (_, mut reader) = fixture.vault.transfers_download_reader(reader);

        let mut content = String::new();
        let res = reader.reader.read_to_string(&mut content).await;
        assert!(matches!(res, Err(std::io::Error { .. })));

        drop(watcher);

        check_recorded(
            recorder,
            |len| assert_eq!(len, 3),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                2 => assert_eq!(transfers, expected_tranfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    });
}

fn expected_transfers_transferring(transfers: &TransfersState, attempts: usize) -> TransfersState {
    TransfersState {
        transfers: [(
            0,
            Transfer {
                id: 0,
                typ: TransferType::DownloadReader,
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: Some(
                    transfers
                        .transfers
                        .get(&0)
                        .and_then(|t| t.started)
                        .unwrap_or(9999),
                ),
                is_persistent: false,
                is_retriable: false,
                state: TransferState::Transferring,
                transferred_bytes: 0,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: 1,
        started: Some(transfers.started.unwrap_or(999)),
        last_progress_update: transfers.last_progress_update,
        transferring_count: 1,
        transferring_uploads_count: 0,
        transferring_downloads_count: 1,
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
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            0,
            Transfer {
                id: 0,
                typ: TransferType::DownloadReader,
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: Some(
                    transfers
                        .transfers
                        .get(&0)
                        .and_then(|t| t.started)
                        .unwrap_or(9999),
                ),
                is_persistent: false,
                is_retriable: false,
                state: TransferState::Transferring,
                transferred_bytes: 4,
                attempts,
                order: 0,
            },
        )]
        .into(),
        next_id: 1,
        started: Some(transfers.started.unwrap_or(999)),
        last_progress_update: transfers.last_progress_update,
        transferring_count: 1,
        transferring_uploads_count: 0,
        transferring_downloads_count: 1,
        done_count: 0,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 4,
        failed_bytes: 0,
        total_bytes: 4,
    }
}

fn expected_tranfers_done() -> TransfersState {
    TransfersState {
        next_id: 1,
        ..Default::default()
    }
}