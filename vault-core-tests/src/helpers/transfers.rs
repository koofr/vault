use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use axum::{
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use futures::{
    channel::oneshot::{self, Sender},
    future::{self, BoxFuture},
    io::Cursor,
    FutureExt,
};
use vault_core::{
    common::state::{BoxAsyncRead, BoxAsyncWrite, SizeInfo},
    store,
    transfers::{
        downloadable::{BoxDownloadable, Downloadable, DownloadableStatus},
        errors::{DownloadableError, UploadableError},
        state::{CreateDownloadResultFuture, Transfer, TransferState, TransfersState},
        uploadable::{BoxUploadable, Uploadable},
    },
    utils::memory_writer::MemoryWriter,
    Vault,
};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;
use vault_store::test_helpers::{StateRecorder, StoreWatcher};

use crate::{fake_remote::FakeRemote, fixtures::repo_fixture::RepoFixture};

use super::with_repo;

pub fn with_transfers(
    f: impl FnOnce(Arc<RepoFixture>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
) {
    with_repo(|repo_fixture| {
        async move {
            repo_fixture.vault.store.mutate(|state, _, _, _| {
                // the default value 100 ms causes flaky tests
                state.config.transfers.progress_throttle = Duration::from_secs(5);
            });

            f(repo_fixture).await;
        }
        .boxed()
    });
}

pub struct TestUploadable {
    pub size_fn: Box<
        dyn Fn() -> BoxFuture<'static, Result<SizeInfo, UploadableError>> + Send + Sync + 'static,
    >,
    pub is_retriable_fn:
        Box<dyn Fn() -> BoxFuture<'static, Result<bool, UploadableError>> + Send + Sync + 'static>,
    pub reader_fn: Box<
        dyn Fn() -> BoxFuture<'static, Result<(BoxAsyncRead, SizeInfo), UploadableError>>
            + Send
            + Sync
            + 'static,
    >,
}

impl TestUploadable {
    pub fn bytes(bytes: Vec<u8>) -> BoxUploadable {
        let bytes = Arc::new(bytes);
        let get_reader_bytes = bytes.clone();

        Box::new(Self {
            size_fn: Box::new(move || {
                future::ready(Ok(SizeInfo::Exact(bytes.len() as i64))).boxed()
            }),
            is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
            reader_fn: Box::new(move || {
                let bytes = get_reader_bytes.clone();

                future::ready(Ok((
                    Box::pin(Cursor::new((*bytes).clone())) as BoxAsyncRead,
                    SizeInfo::Exact(bytes.len() as i64),
                )))
                .boxed()
            }),
        })
    }

    pub fn string(s: &str) -> BoxUploadable {
        Self::bytes(s.as_bytes().to_vec())
    }
}

#[async_trait]
impl Uploadable for TestUploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError> {
        (self.size_fn)().await
    }

    async fn is_retriable(&self) -> Result<bool, UploadableError> {
        (self.is_retriable_fn)().await
    }

    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError> {
        (self.reader_fn)().await
    }
}

pub struct TestDownloadable {
    pub is_retriable_fn: Box<
        dyn Fn() -> BoxFuture<'static, Result<bool, DownloadableError>> + Send + Sync + 'static,
    >,
    pub exists_fn: Box<
        dyn Fn(String, String) -> BoxFuture<'static, Result<bool, DownloadableError>>
            + Send
            + Sync
            + 'static,
    >,
    pub writer_fn: Box<
        dyn Fn(
                String,
                SizeInfo,
                Option<String>,
                Option<String>,
            ) -> BoxFuture<'static, Result<BoxAsyncWrite, DownloadableError>>
            + Send
            + Sync
            + 'static,
    >,
    pub done_fn: Box<
        dyn Fn(
                Result<DownloadableStatus, DownloadableError>,
            ) -> BoxFuture<'static, Result<(), DownloadableError>>
            + Send
            + Sync
            + 'static,
    >,
    pub sender: Arc<Mutex<Option<Sender<Option<Vec<u8>>>>>>,
}

impl TestDownloadable {
    pub fn bytes() -> (BoxDownloadable, BoxFuture<'static, Option<Vec<u8>>>) {
        let data: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));

        let writer_data = data.clone();

        let (sender, receiver) = oneshot::channel::<Option<Vec<u8>>>();

        let sender = Arc::new(Mutex::new(Some(sender)));

        let done_sender = sender.clone();

        let downloadable = Box::new(TestDownloadable {
            is_retriable_fn: Box::new(|| future::ready(Ok(true)).boxed()),
            exists_fn: Box::new(|_, _| future::ready(Ok(false)).boxed()),
            writer_fn: Box::new(move |_, _, _, _| {
                let data = writer_data.clone();

                future::ready(Ok(Box::pin(MemoryWriter::new(Box::new(move |buf| {
                    *data.lock().unwrap() = Some(buf);
                }))) as BoxAsyncWrite))
                .boxed()
            }),
            done_fn: Box::new(move |res| {
                match res {
                    Ok(DownloadableStatus::Downloaded) => {
                        let data = data.lock().unwrap().take();

                        if let Some(sender) = done_sender.lock().unwrap().take() {
                            let _ = sender.send(data);
                        }
                    }
                    Ok(DownloadableStatus::AlreadyExists) => {}
                    Err(_) => {}
                }

                future::ready(Ok(())).boxed()
            }),
            sender,
        });

        (downloadable, async { receiver.await.unwrap() }.boxed())
    }

    pub fn string() -> (BoxDownloadable, BoxFuture<'static, Option<String>>) {
        let (downloadable, future) = Self::bytes();

        let future = async { future.await.map(|buf| String::from_utf8(buf).unwrap()) }.boxed();

        (downloadable, future)
    }
}

#[async_trait]
impl Downloadable for TestDownloadable {
    async fn is_retriable(&self) -> Result<bool, DownloadableError> {
        (self.is_retriable_fn)().await
    }

    async fn exists(
        &mut self,
        name: String,
        unique_name: String,
    ) -> Result<bool, DownloadableError> {
        (self.exists_fn)(name, unique_name).await
    }

    async fn writer(
        &mut self,
        name: String,
        size: SizeInfo,
        content_type: Option<String>,
        unique_name: Option<String>,
    ) -> Result<BoxAsyncWrite, DownloadableError> {
        (self.writer_fn)(name, size, content_type, unique_name).await
    }

    async fn done(
        &self,
        res: Result<DownloadableStatus, DownloadableError>,
    ) -> Result<(), DownloadableError> {
        (self.done_fn)(res).await
    }
}

impl Drop for TestDownloadable {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.lock().unwrap().take() {
            let _ = sender.send(None);
        }
    }
}

pub fn download_string(
    vault: &Vault,
    repo_id: &str,
    path: &str,
) -> (
    u32,
    CreateDownloadResultFuture,
    BoxFuture<'static, Option<String>>,
) {
    let reader_provider = vault.repo_files_get_file_reader(repo_id, path).unwrap();
    let (downloadable, content_future) = TestDownloadable::string();
    let (id, future) = vault.transfers_download(reader_provider, downloadable);

    (id, future, content_future)
}

pub fn download_delay_response_body(fake_remote: &FakeRemote, duration: Duration) {
    fake_remote.intercept(Box::new(move |parts| {
        if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/get") {
            InterceptorResult::delayed_response_body(duration)
        } else {
            InterceptorResult::Ignore
        }
    }));
}

pub fn patch_transfer<Patch: Fn(&mut Transfer)>(
    mut transfers: TransfersState,
    transfer_id: u32,
    patch: Patch,
) -> TransfersState {
    match transfers.transfers.get_mut(&transfer_id) {
        Some(transfer) => patch(transfer),
        None => panic!("Transfer {} not found", transfer_id),
    }

    transfers
}

pub fn transfers_recorder(vault: &Vault) -> StateRecorder<TransfersState> {
    StateRecorder::record(vault.store.clone(), &[store::Event::Transfers], |state| {
        state.transfers.clone()
    })
}

pub fn check_recorded(
    recorder: StateRecorder<TransfersState>,
    check_len: impl FnOnce(usize),
    check_transfers: impl Fn(usize, TransfersState),
) {
    let entries = recorder.collect_enumerated();
    let entries_len = entries.len();

    for (i, transfers) in entries {
        check_transfers(i, transfers);
    }

    // check len at the end so that we get more useful asserts of what is different
    check_len(entries_len);
}

pub fn transfer_do_when<
    Filter: Fn(&Transfer) -> bool + Send + Sync + 'static,
    Action: Fn(&Vault) + Send + Sync + 'static,
>(
    vault: Arc<Vault>,
    transfer_id: u32,
    filter: Filter,
    action: Action,
) -> StoreWatcher {
    let action_vault = vault.clone();

    StoreWatcher::watch_store(
        vault.store.clone(),
        &[store::Event::Transfers],
        move |store, _| {
            if store.with_state(|state| {
                state
                    .transfers
                    .transfers
                    .get(&transfer_id)
                    .filter(|t| filter(&t))
                    .is_some()
            }) {
                action(&action_vault);
            }
        },
    )
}

pub fn transfer_abort_when<Filter: Fn(&Transfer) -> bool + Send + Sync + 'static>(
    vault: Arc<Vault>,
    transfer_id: u32,
    filter: Filter,
) -> StoreWatcher {
    transfer_do_when(vault, transfer_id, filter, move |vault| {
        vault.transfers_abort(transfer_id);
    })
}

pub async fn transfer_wait<Filter: Fn(&Transfer) -> bool + Send + Sync + 'static>(
    store: Arc<store::Store>,
    transfer_id: u32,
    filter: Filter,
) {
    store::wait_for(store.clone(), &[store::Event::Transfers], move || {
        store.with_state(|state| {
            state
                .transfers
                .transfers
                .get(&transfer_id)
                .filter(|t| filter(t))
                .map(|_| ())
        })
    })
    .await;
}

pub fn uploaded_server_error(
    store: Arc<store::Store>,
    transfer_id: u32,
    bytes: i64,
) -> InterceptorResult {
    InterceptorResult::AsyncResponse(
        async move {
            transfer_wait(store, transfer_id, move |t| {
                matches!(t.state, TransferState::Transferring) && t.transferred_bytes == bytes
            })
            .await;

            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        .boxed(),
    )
}

pub fn capture_upload_uri(fake_remote: &FakeRemote) -> oneshot::Receiver<Uri> {
    let (upload_uri_sender, upload_uri_receiver) = oneshot::channel();
    let interceptor_upload_uri_sender = Arc::new(Mutex::new(Some(upload_uri_sender)));

    fake_remote.intercept(Box::new(move |parts| {
        if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/put") {
            if let Some(sender) = interceptor_upload_uri_sender.lock().unwrap().take() {
                let _ = sender.send(parts.uri.to_owned());
            }
        }

        InterceptorResult::Ignore
    }));

    upload_uri_receiver
}
