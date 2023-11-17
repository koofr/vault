use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use futures::{future, FutureExt};
use similar_asserts::assert_eq;

use vault_core::{
    common::state::{BoxAsyncWrite, SizeInfo},
    files::file_category::FileCategory,
    http::HttpError,
    remote::RemoteError,
    repo_files_read::{errors::GetFilesReaderError, state::RepoFileReaderProvider},
    store::NextId,
    transfers::{
        errors::{DownloadableError, TransferError},
        state::{DownloadTransfer, Transfer, TransferState, TransferType, TransfersState},
    },
    types::{DecryptedName, DecryptedPath},
    utils::memory_writer::MemoryWriter,
};
use vault_core_tests::helpers::transfers::{
    download_delay_response_body, download_string, patch_transfer, transfer_abort_when,
    transfer_do_when, transfers_recorder, with_transfers, TestDownloadable,
};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;

#[test]
fn test_download() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            assert!(matches!(future.await.unwrap(), ()));
            assert_eq!(content_future.await.unwrap(), "test");

            recorder.check_recorded(
                |len| assert_eq!(len, 6),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                    4 => assert_eq!(
                        transfers,
                        expected_transfers_transferring_progress(&transfers, 1)
                    ),
                    5 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_change_name() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let reader_provider = fixture
                .vault
                .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
                .unwrap()
                .wrap_reader_builder(|reader_builder| {
                    async move {
                        reader_builder().await.map(|mut reader| {
                            reader.name = DecryptedName("file renamed.txt".into());
                            reader
                        })
                    }
                    .boxed()
                });
            let (downloadable, content_future) = TestDownloadable::string();
            let (_, create_future) = fixture
                .vault
                .transfers_download(reader_provider, Box::new(downloadable));
            let future = create_future.await.unwrap();

            assert!(matches!(future.await.unwrap(), ()));
            assert_eq!(content_future.await.unwrap(), "test");

            recorder.check_recorded(
                |len| assert_eq!(len, 6),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(
                        transfers,
                        patch_transfer(expected_transfers_transferring(&transfers, 1), 1, |t| {
                            t.typ = TransferType::Download(DownloadTransfer {
                                name: "file renamed.txt".into(),
                            });
                            t.name = "file renamed.txt".into();
                        })
                    ),
                    4 => assert_eq!(
                        transfers,
                        patch_transfer(
                            expected_transfers_transferring_progress(&transfers, 1),
                            1,
                            |t| {
                                t.typ = TransferType::Download(DownloadTransfer {
                                    name: "file renamed.txt".into(),
                                });
                                t.name = "file renamed.txt".into();
                            }
                        )
                    ),
                    5 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_change_name_writer() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let reader_provider = fixture
                .vault
                .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
                .unwrap();
            let (mut downloadable, content_future) = TestDownloadable::string();
            let writer_data = downloadable.data.clone();
            downloadable.writer_fn = Box::new(move |_, _, _, _| {
                let data = writer_data.clone();

                let writer: BoxAsyncWrite = Box::pin(MemoryWriter::new(Box::new(move |buf| {
                    *data.lock().unwrap() = Some(buf);
                })));

                future::ready(Ok((writer, "file renamed.txt".into()))).boxed()
            });
            let (_, create_future) = fixture
                .vault
                .transfers_download(reader_provider, Box::new(downloadable));
            let future = create_future.await.unwrap();

            assert!(matches!(future.await.unwrap(), ()));
            assert_eq!(content_future.await.unwrap(), "test");

            recorder.check_recorded(
                |len| assert_eq!(len, 6),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(
                        transfers,
                        patch_transfer(expected_transfers_transferring(&transfers, 1), 1, |t| {
                            t.typ = TransferType::Download(DownloadTransfer {
                                name: "file renamed.txt".into(),
                            });
                            t.name = "file renamed.txt".into();
                        })
                    ),
                    4 => assert_eq!(
                        transfers,
                        patch_transfer(
                            expected_transfers_transferring_progress(&transfers, 1),
                            1,
                            |t| {
                                t.typ = TransferType::Download(DownloadTransfer {
                                    name: "file renamed.txt".into(),
                                });
                                t.name = "file renamed.txt".into();
                            }
                        )
                    ),
                    5 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_already_exists() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let reader_provider = fixture
                .vault
                .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
                .unwrap();
            let downloadable = Box::new(TestDownloadable {
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                is_openable_fn: Box::new(|| future::ready(Ok(false)).boxed()),
                exists_fn: Box::new(|_, _| future::ready(Ok(true)).boxed()),
                writer_fn: Box::new(|_, _, _, _| panic!("unreachable")),
                done_fn: Box::new(|_| future::ready(Ok(())).boxed()),
                open_fn: Box::new(|| future::ready(Ok(())).boxed()),
                sender: Default::default(),
                data: Default::default(),
            });
            let (_, create_future) = fixture
                .vault
                .transfers_download(reader_provider, downloadable);

            assert!(matches!(
                create_future.await,
                Err(TransferError::AlreadyExists)
            ));

            recorder.check_recorded(
                |len| assert_eq!(len, 1),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_already_exists_error() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let reader_provider = fixture
                .vault
                .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
                .unwrap();
            let downloadable = Box::new(TestDownloadable {
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                is_openable_fn: Box::new(|| future::ready(Ok(false)).boxed()),
                exists_fn: Box::new(|_, _| {
                    future::ready(Err(DownloadableError::LocalFileError("io error".into()))).boxed()
                }),
                writer_fn: Box::new(|_, _, _, _| panic!("unreachable")),
                done_fn: Box::new(|_| future::ready(Ok(())).boxed()),
                open_fn: Box::new(|| future::ready(Ok(())).boxed()),
                sender: Default::default(),
                data: Default::default(),
            });
            let (_, create_future) = fixture
                .vault
                .transfers_download(reader_provider, downloadable);

            assert!(matches!(
                create_future.await,
                Err(TransferError::LocalFileError(err)) if err == "io error"
            ));

            recorder.check_recorded(
                |len| assert_eq!(len, 1),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_already_exists_done_error() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let reader_provider = fixture
                .vault
                .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
                .unwrap();
            let downloadable = Box::new(TestDownloadable {
                is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
                is_openable_fn: Box::new(|| future::ready(Ok(false)).boxed()),
                exists_fn: Box::new(|_, _| future::ready(Ok(true)).boxed()),
                writer_fn: Box::new(|_, _, _, _| panic!("unreachable")),
                done_fn: Box::new(|_| {
                    future::ready(Err(DownloadableError::LocalFileError("done error".into())))
                        .boxed()
                }),
                open_fn: Box::new(|| future::ready(Ok(())).boxed()),
                sender: Default::default(),
                data: Default::default(),
            });
            let (_, create_future) = fixture
                .vault
                .transfers_download(reader_provider, downloadable);

            assert!(matches!(
                create_future.await,
                Err(TransferError::LocalFileError(err)) if err == "done error"
            ));

            recorder.check_recorded(
                |len| assert_eq!(len, 1),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_reader_error() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
                matches!(t.state, TransferState::Failed { .. })
            });

            let reader_provider = RepoFileReaderProvider {
                name: DecryptedName("file.txt".into()),
                size: SizeInfo::Exact(4),
                unique_name: None,
                reader_builder: Box::new(|| {
                    async {
                        Err(GetFilesReaderError::RemoteError(RemoteError::HttpError(
                            HttpError::ResponseError("response error".into()),
                        )))
                    }
                    .boxed()
                }),
            };
            let (downloadable, content_future) = TestDownloadable::string();
            let (_, create_future) = fixture
                .vault
                .transfers_download(reader_provider, Box::new(downloadable));
            let future = create_future.await.unwrap();

            assert!(matches!(future.await, Err(TransferError::Aborted)));
            assert!(matches!(content_future.await, None));

            drop(watcher);

            recorder.check_recorded(
                |len| assert_eq!(len, 13),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                    4 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                    5 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 2)),
                    6 => assert_eq!(transfers, expected_transfers_processing(&transfers, 3)),
                    7 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 3)),
                    8 => assert_eq!(transfers, expected_transfers_processing(&transfers, 4)),
                    9 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 4)),
                    10 => assert_eq!(transfers, expected_transfers_processing(&transfers, 5)),
                    11 => {
                        assert_eq!(transfers, expected_transfers_failed(&transfers, 5));
                        match &transfers.transfers.get(&1).as_ref().unwrap().state {
                            TransferState::Failed { error } => assert!(
                                matches!(error, TransferError::RemoteError(RemoteError::HttpError(
                                HttpError::ResponseError(err),
                            )) if err == "response error")
                            ),
                            _ => {}
                        }
                    }
                    12 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_downloadable_writer_error() {
    with_transfers(|fixture| {
        async move {
        fixture.upload_file("/file.txt", "test").await;

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Failed { .. })
        });

        let reader_provider = fixture
            .vault
            .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
            .unwrap();
        let downloadable = Box::new(TestDownloadable {
            is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
            is_openable_fn: Box::new(|| future::ready(Ok(false)).boxed()),
            exists_fn: Box::new(|_, _| future::ready(Ok(false)).boxed()),
            writer_fn: Box::new(|_, _, _, _| {
                future::ready(Err(DownloadableError::LocalFileError(
                    "writer error".into(),
                )))
                .boxed()
            }),
            done_fn: Box::new(|_| future::ready(Ok(())).boxed()),
            open_fn: Box::new(|| future::ready(Ok(())).boxed()),
            sender: Default::default(),
            data: Default::default(),
        });
        let (_, create_future) = fixture
            .vault
            .transfers_download(reader_provider, downloadable);
        let future = create_future.await.unwrap();

        assert!(matches!(future.await, Err(TransferError::Aborted)));

        drop(watcher);

        recorder.check_recorded(
            |len| assert_eq!(len, 13),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                3 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                4 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                5 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 2)),
                6 => assert_eq!(transfers, expected_transfers_processing(&transfers, 3)),
                7 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 3)),
                8 => assert_eq!(transfers, expected_transfers_processing(&transfers, 4)),
                9 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 4)),
                10 => assert_eq!(transfers, expected_transfers_processing(&transfers, 5)),
                11 => {
                    assert_eq!(transfers, expected_transfers_failed(&transfers, 5));
                    match &transfers.transfers.get(&1).as_ref().unwrap().state {
                        TransferState::Failed { error } => assert!(
                            matches!(error, TransferError::LocalFileError(err) if err == "writer error")
                        ),
                        _ => {}
                    }
                }
                12 => assert_eq!(transfers, expected_transfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    }.boxed()
    });
}

#[test]
fn test_download_downloadable_close_error() {
    with_transfers(|fixture| {
        async move {
        fixture.upload_file("/file.txt", "test").await;

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Failed { .. })
        });

        let reader_provider = fixture
            .vault
            .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
            .unwrap();

        use futures::AsyncWrite;
        use std::{
            io::{Error, ErrorKind, Result},
            pin::Pin,
            task::{Context, Poll},
        };

        pub struct CloseErrorWriter;

        impl AsyncWrite for CloseErrorWriter {
            fn poll_write(
                self: Pin<&mut Self>,
                _: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<Result<usize>> {
                Poll::Ready(Ok(buf.len()))
            }

            fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
                Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, "close error")))
            }
        }

        let downloadable = Box::new(TestDownloadable {
            is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
            is_openable_fn: Box::new(|| future::ready(Ok(false)).boxed()),
            exists_fn: Box::new(|_, _| future::ready(Ok(false)).boxed()),
            writer_fn: Box::new(|name, _, _, _| {
                future::ready(Ok((Box::pin(CloseErrorWriter) as BoxAsyncWrite, name))).boxed()
            }),
            done_fn: Box::new(|_| future::ready(Ok(())).boxed()),
            open_fn: Box::new(|| future::ready(Ok(())).boxed()),
            sender: Default::default(),
            data: Default::default(),
        });
        let (_, create_future) = fixture
            .vault
            .transfers_download(reader_provider, downloadable);
        let future = create_future.await.unwrap();

        assert!(matches!(future.await, Err(TransferError::Aborted)));

        drop(watcher);

        recorder.check_recorded(
            |len| assert_eq!(len, 19),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&transfers, 1)
                ),
                5 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                6 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                7 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 2)),
                8 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 2)),
                9 => assert_eq!(transfers, expected_transfers_processing(&transfers, 3)),
                10 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 3)),
                11 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 3)),
                12 => assert_eq!(transfers, expected_transfers_processing(&transfers, 4)),
                13 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 4)),
                14 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 4)),
                15 => assert_eq!(transfers, expected_transfers_processing(&transfers, 5)),
                16 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 5)),
                17 => {
                    assert_eq!(transfers, expected_transfers_failed(&transfers, 5));
                    match &transfers.transfers.get(&1).as_ref().unwrap().state {
                        TransferState::Failed { error } => assert!(
                            matches!(error, TransferError::LocalFileError(err) if err == "close error")
                        ),
                        _ => {}
                    }
                }
                18 => assert_eq!(transfers, expected_transfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    }.boxed()
    });
}

#[test]
fn test_download_downloadable_done_error() {
    with_transfers(|fixture| {
        async move {
        fixture.upload_file("/file.txt", "test").await;

        let recorder = transfers_recorder(&fixture.vault);

        let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
            matches!(t.state, TransferState::Failed { .. })
        });

        let reader_provider = fixture
            .vault
            .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
            .unwrap();
        let downloadable = Box::new(TestDownloadable {
            is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
            is_openable_fn: Box::new(|| future::ready(Ok(false)).boxed()),
            exists_fn: Box::new(|_, _| future::ready(Ok(false)).boxed()),
            writer_fn: Box::new(|name, _, _, _| {
                future::ready(Ok(
                    (Box::pin(MemoryWriter::new(Box::new(|_| {}))) as BoxAsyncWrite, name)
                ))
                .boxed()
            }),
            done_fn: Box::new(|_| {
                future::ready(Err(DownloadableError::LocalFileError("done error".into()))).boxed()
            }),
            open_fn: Box::new(|| future::ready(Ok(())).boxed()),
            sender: Default::default(),
            data: Default::default(),
        });
        let (_, create_future) = fixture
            .vault
            .transfers_download(reader_provider, downloadable);
        let future = create_future.await.unwrap();

        assert!(matches!(future.await, Err(TransferError::Aborted)));

        drop(watcher);

        recorder.check_recorded(
            |len| assert_eq!(len, 19),
            |i, transfers| match i {
                0 => assert_eq!(transfers, TransfersState::default()),
                1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                4 => assert_eq!(
                    transfers,
                    expected_transfers_transferring_progress(&transfers, 1)
                ),
                5 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                6 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                7 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 2)),
                8 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 2)),
                9 => assert_eq!(transfers, expected_transfers_processing(&transfers, 3)),
                10 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 3)),
                11 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 3)),
                12 => assert_eq!(transfers, expected_transfers_processing(&transfers, 4)),
                13 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 4)),
                14 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 4)),
                15 => assert_eq!(transfers, expected_transfers_processing(&transfers, 5)),
                16 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 5)),
                17 => {
                    assert_eq!(transfers, expected_transfers_failed(&transfers, 5));
                    match &transfers.transfers.get(&1).as_ref().unwrap().state {
                        TransferState::Failed { error } => assert!(
                            matches!(error, TransferError::LocalFileError(err) if err == "done error")
                        ),
                        _ => {}
                    }
                }
                18 => assert_eq!(transfers, expected_transfers_done()),
                _ => panic!("unexpected state: {:#?}", transfers),
            },
        );
    }.boxed()
    });
}

#[test]
fn test_download_abort_immediately() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let (transfer_id, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            fixture.vault.transfers_abort(transfer_id);

            assert!(matches!(create_future.await, Err(TransferError::Aborted)));
            assert!(matches!(content_future.await, None));

            recorder.check_recorded(
                |len| assert_eq!(len, 1),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_abort_waiting() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
                matches!(t.state, TransferState::Waiting)
            });

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            assert!(matches!(future.await, Err(TransferError::Aborted)));
            assert!(matches!(content_future.await, None));

            drop(watcher);

            recorder.check_recorded(
                |len| assert_eq!(len, 3),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_abort_processing() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
                matches!(t.state, TransferState::Processing)
            });

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            assert!(matches!(future.await, Err(TransferError::Aborted)));
            assert!(matches!(content_future.await, None));

            drop(watcher);

            recorder.check_recorded(
                |len| assert_eq!(len, 4),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_abort_transferring() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            download_delay_response_body(&fixture.fake_remote, Duration::from_millis(50));

            let recorder = transfers_recorder(&fixture.vault);

            let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
                matches!(t.state, TransferState::Transferring)
            });

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            assert!(matches!(future.await, Err(TransferError::Aborted)));
            assert!(matches!(content_future.await, None));

            drop(watcher);

            recorder.check_recorded(
                |len| assert_eq!(len, 5),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                    4 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_fail_autoretry_succeed() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let download_counter = Arc::new(AtomicUsize::new(0));
            let interceptor_download_counter = download_counter.clone();

            fixture.fake_remote.intercept(Box::new(move |parts| {
                if parts.uri.path().contains("/content/api")
                    && parts.uri.path().contains("/files/get")
                {
                    if interceptor_download_counter.fetch_add(1, Ordering::SeqCst) == 0 {
                        InterceptorResult::delayed_abort_response_body(Duration::from_millis(50))
                    } else {
                        InterceptorResult::Ignore
                    }
                } else {
                    InterceptorResult::Ignore
                }
            }));

            let recorder = transfers_recorder(&fixture.vault);

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            assert!(matches!(future.await.unwrap(), ()));
            assert_eq!(content_future.await.unwrap(), "test");

            recorder.check_recorded(
                |len| assert_eq!(len, 9),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                    4 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                    5 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                    6 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 2)),
                    7 => assert_eq!(
                        transfers,
                        expected_transfers_transferring_progress(&transfers, 2)
                    ),
                    8 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_fail_autoretry_fail() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            fixture.fake_remote.intercept(Box::new(move |parts| {
                if parts.uri.path().contains("/content/api")
                    && parts.uri.path().contains("/files/get")
                {
                    InterceptorResult::delayed_abort_response_body(Duration::from_millis(50))
                } else {
                    InterceptorResult::Ignore
                }
            }));

            let recorder = transfers_recorder(&fixture.vault);

            let watcher = transfer_abort_when(fixture.vault.clone(), 1, |t| {
                matches!(t.state, TransferState::Failed { .. })
            });

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            let res = future.await;
            // TODO should this be Aborted or the last error from the Failed transfer
            assert!(matches!(res, Err(TransferError::Aborted)));
            assert!(matches!(content_future.await, None));

            drop(watcher);

            recorder.check_recorded(
                |len| assert_eq!(len, 18),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                    4 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                    5 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                    6 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 2)),
                    7 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 2)),
                    8 => assert_eq!(transfers, expected_transfers_processing(&transfers, 3)),
                    9 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 3)),
                    10 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 3)),
                    11 => assert_eq!(transfers, expected_transfers_processing(&transfers, 4)),
                    12 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 4)),
                    13 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 4)),
                    14 => assert_eq!(transfers, expected_transfers_processing(&transfers, 5)),
                    15 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 5)),
                    16 => assert_eq!(transfers, expected_transfers_failed(&transfers, 5)),
                    17 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_fail_autoretry_retry() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let download_counter = Arc::new(AtomicUsize::new(0));
            let interceptor_download_counter = download_counter.clone();

            fixture.fake_remote.intercept(Box::new(move |parts| {
                if parts.uri.path().contains("/content/api")
                    && parts.uri.path().contains("/files/get")
                {
                    if interceptor_download_counter.fetch_add(1, Ordering::SeqCst) < 5 {
                        InterceptorResult::delayed_abort_response_body(Duration::from_millis(50))
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

            let (_, create_future, content_future) =
                download_string(&fixture.vault, &fixture.repo_id.0, "/file.txt");
            let future = create_future.await.unwrap();

            assert!(matches!(future.await.unwrap(), ()));
            assert_eq!(content_future.await.unwrap(), "test");

            drop(watcher);

            recorder.check_recorded(
                |len| assert_eq!(len, 22),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(transfers, expected_transfers_waiting(&transfers)),
                    2 => assert_eq!(transfers, expected_transfers_processing(&transfers, 1)),
                    3 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 1)),
                    4 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 1)),
                    5 => assert_eq!(transfers, expected_transfers_processing(&transfers, 2)),
                    6 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 2)),
                    7 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 2)),
                    8 => assert_eq!(transfers, expected_transfers_processing(&transfers, 3)),
                    9 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 3)),
                    10 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 3)),
                    11 => assert_eq!(transfers, expected_transfers_processing(&transfers, 4)),
                    12 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 4)),
                    13 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 4)),
                    14 => assert_eq!(transfers, expected_transfers_processing(&transfers, 5)),
                    15 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 5)),
                    16 => assert_eq!(transfers, expected_transfers_failed(&transfers, 5)),
                    17 => assert_eq!(transfers, expected_transfers_waiting_failed(&transfers, 5)),
                    18 => assert_eq!(transfers, expected_transfers_processing(&transfers, 6)),
                    19 => assert_eq!(transfers, expected_transfers_transferring(&transfers, 6)),
                    20 => assert_eq!(
                        transfers,
                        expected_transfers_transferring_progress(&transfers, 6)
                    ),
                    21 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

#[test]
fn test_download_openable() {
    with_transfers(|fixture| {
        async move {
            fixture.upload_file("/file.txt", "test").await;

            let recorder = transfers_recorder(&fixture.vault);

            let reader_provider = fixture
                .vault
                .repo_files_get_file_reader(&fixture.repo_id, &DecryptedPath("/file.txt".into()))
                .unwrap();
            let (mut downloadable, content_future) = TestDownloadable::string();
            downloadable.is_openable_fn = Box::new(|| future::ready(Ok(true)).boxed());
            let opened = Arc::new(AtomicBool::new(false));
            let downloadable_opened = opened.clone();
            downloadable.open_fn = Box::new(move || {
                downloadable_opened.store(true, Ordering::SeqCst);

                future::ready(Ok(())).boxed()
            });
            let (transfer_id, create_future) = fixture
                .vault
                .transfers_download(reader_provider, Box::new(downloadable));
            let future = create_future.await.unwrap();

            assert!(matches!(future.await.unwrap(), ()));
            assert_eq!(content_future.await.unwrap(), "test");

            assert!(!opened.load(Ordering::SeqCst));

            fixture.vault.transfers_open(transfer_id).await.unwrap();

            assert!(opened.load(Ordering::SeqCst));

            fixture.vault.transfers_abort(transfer_id);

            let patch = |t: &mut Transfer| {
                t.is_persistent = true;
                t.is_openable = true;
            };

            recorder.check_recorded(
                |len| assert_eq!(len, 7),
                |i, transfers| match i {
                    0 => assert_eq!(transfers, TransfersState::default()),
                    1 => assert_eq!(
                        transfers,
                        patch_transfer(expected_transfers_waiting(&transfers), 1, patch)
                    ),
                    2 => assert_eq!(
                        transfers,
                        patch_transfer(expected_transfers_processing(&transfers, 1), 1, patch)
                    ),
                    3 => assert_eq!(
                        transfers,
                        patch_transfer(expected_transfers_transferring(&transfers, 1), 1, patch)
                    ),
                    4 => assert_eq!(
                        transfers,
                        patch_transfer(
                            expected_transfers_transferring_progress(&transfers, 1),
                            1,
                            patch
                        )
                    ),
                    5 => assert_eq!(transfers, expected_transfers_done_openable(&transfers, 1)),
                    6 => assert_eq!(transfers, expected_transfers_done()),
                    _ => panic!("unexpected state: {:#?}", transfers),
                },
            );
        }
        .boxed()
    });
}

fn expected_transfers_waiting(transfers: &TransfersState) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
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

fn expected_transfers_processing(transfers: &TransfersState, attempts: usize) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
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
                is_openable: false,
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

fn expected_transfers_transferring(transfers: &TransfersState, attempts: usize) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
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
                is_openable: false,
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
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
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
                is_openable: false,
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

fn expected_transfers_waiting_failed(
    transfers: &TransfersState,
    attempts: usize,
) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
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

fn expected_transfers_failed(transfers: &TransfersState, attempts: usize) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
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

fn expected_transfers_done() -> TransfersState {
    TransfersState {
        next_id: NextId(2),
        ..Default::default()
    }
}

fn expected_transfers_done_openable(transfers: &TransfersState, attempts: usize) -> TransfersState {
    TransfersState {
        transfers: [(
            1,
            Transfer {
                id: 1,
                typ: TransferType::Download(DownloadTransfer {
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(4),
                category: FileCategory::Text,
                started: None,
                is_persistent: true,
                is_retriable: true,
                is_openable: true,
                state: TransferState::Done,
                transferred_bytes: 4,
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
        done_count: 1,
        failed_count: 0,
        retriable_count: 0,
        total_count: 1,
        done_bytes: 4,
        failed_bytes: 0,
        total_bytes: 4,
    }
}
