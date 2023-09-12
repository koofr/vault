use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::{
    channel::oneshot::{self, Sender},
    io::{self, BufReader},
    stream::{AbortHandle, AbortRegistration, Abortable, Aborted},
    AsyncWriteExt, FutureExt,
};

use crate::{
    common::state::SizeInfo,
    remote::ApiErrorCode,
    repo_files::{
        errors::LoadFilesError, selectors as repo_files_selectors,
        state::RepoFilesUploadConflictResolution, RepoFilesService,
    },
    repo_files_read::state::{RepoFileReader, RepoFileReaderProvider},
    runtime, store,
    utils::{
        abort_reader::AbortReader, on_end_reader::OnEndReader, progress_reader::ProgressReader,
    },
};

use super::{
    downloadable::{BoxDownloadable, DownloadableStatus},
    errors::{DownloadableError, TransferError},
    mutations, selectors,
    state::{
        CreateDownloadResult, CreateDownloadResultFuture, CreateUploadResult,
        CreateUploadResultFuture, DownloadReaderResult, DownloadResult, RetryInitiator,
        TransferType, UploadResult, UploadTransfer,
    },
    uploadable::BoxUploadable,
};

#[derive(Default)]
struct TransfersServiceTransferStateUpload {
    uploadable: Option<Arc<BoxUploadable>>,
    result_sender: Option<Sender<UploadResult>>,
}

#[derive(Default)]
struct TransfersServiceTransferStateDownload {
    reader_provider: Option<Arc<RepoFileReaderProvider>>,
    downloadable: Option<Arc<futures::lock::Mutex<BoxDownloadable>>>,
    result_sender: Option<Sender<DownloadResult>>,
}

enum TransfersServiceTransferStateType {
    Upload(TransfersServiceTransferStateUpload),
    Download(TransfersServiceTransferStateDownload),
    DownloadReader,
}

struct TransfersServiceTransferState {
    typ: TransfersServiceTransferStateType,
    abort_handle: Option<AbortHandle>,
}

#[derive(Default)]
struct TransfersServiceState {
    transfers: HashMap<u32, TransfersServiceTransferState>,
}

pub struct TransfersService {
    repo_files_service: Arc<RepoFilesService>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,

    state: Arc<RwLock<TransfersServiceState>>,
}

impl TransfersService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Self {
        Self {
            repo_files_service,
            store,
            runtime,

            state: Default::default(),
        }
    }

    fn get_next_id(&self) -> u32 {
        self.store
            .mutate(|state, _, _, _| mutations::get_next_id(state))
    }

    pub fn upload(
        self: Arc<Self>,
        repo_id: String,
        parent_path: String,
        name: String,
        uploadable: BoxUploadable,
    ) -> (u32, CreateUploadResultFuture) {
        let id = self.get_next_id();

        self.state.write().unwrap().transfers.insert(
            id,
            TransfersServiceTransferState {
                typ: TransfersServiceTransferStateType::Upload(Default::default()),
                abort_handle: None,
            },
        );

        let future = self
            .create_upload(repo_id, parent_path, name, uploadable, id)
            .boxed();

        (id, future)
    }

    async fn create_upload(
        self: Arc<Self>,
        repo_id: String,
        parent_path: String,
        name: String,
        uploadable: BoxUploadable,
        id: u32,
    ) -> CreateUploadResult {
        let size = uploadable.size().await?;

        let is_retriable = uploadable.is_retriable().await?;

        let result_receiver = self.store.mutate(|state, notify, _, _| {
            let result_receiver = match self.state.write().unwrap().transfers.get_mut(&id) {
                Some(state) => {
                    let (result_sender, result_receiver) = oneshot::channel();

                    match &mut state.typ {
                        TransfersServiceTransferStateType::Upload(upload) => {
                            upload.uploadable = Some(Arc::new(uploadable));
                            upload.result_sender = Some(result_sender);
                        }
                        _ => {}
                    }

                    result_receiver
                }
                None => return Err(TransferError::Aborted),
            };

            let is_persistent = false;
            let is_openable = false;

            mutations::create_transfer(
                state,
                notify,
                id,
                mutations::CreateTransferType::Upload {
                    repo_id,
                    parent_path,
                    name,
                },
                size,
                is_persistent,
                is_retriable,
                is_openable,
            );

            Ok(result_receiver)
        })?;

        self.process_next();

        Ok(async { result_receiver.await.unwrap() }.boxed())
    }

    pub fn download(
        self: Arc<Self>,
        reader_provider: RepoFileReaderProvider,
        downloadable: BoxDownloadable,
    ) -> (u32, CreateDownloadResultFuture) {
        let id = self.get_next_id();

        self.state.write().unwrap().transfers.insert(
            id,
            TransfersServiceTransferState {
                typ: TransfersServiceTransferStateType::Download(Default::default()),
                abort_handle: None,
            },
        );

        let future = self
            .create_download(reader_provider, downloadable, id)
            .boxed();

        (id, future)
    }

    async fn create_download(
        self: Arc<Self>,
        reader_provider: RepoFileReaderProvider,
        mut downloadable: BoxDownloadable,
        id: u32,
    ) -> CreateDownloadResult {
        let name = reader_provider.name.clone();
        let size = reader_provider.size;

        if let Some(unique_name) = &reader_provider.unique_name {
            match downloadable
                .exists(name.clone(), unique_name.to_owned())
                .await
            {
                Ok(true) => {
                    return match downloadable
                        .done(Ok(DownloadableStatus::AlreadyExists))
                        .await
                    {
                        Ok(_) => Err(TransferError::AlreadyExists),
                        Err(err) => {
                            return Err(err.into());
                        }
                    }
                }
                Ok(false) => {}
                Err(err) => {
                    let _ = downloadable.done(Err(err.clone())).await;

                    return Err(err.into());
                }
            }
        }

        let is_retriable = downloadable.is_retriable().await?;
        let is_openable = downloadable.is_openable().await?;
        let is_persistent = is_openable;

        let result_receiver = self.store.mutate(|state, notify, _, _| {
            let result_receiver = match self.state.write().unwrap().transfers.get_mut(&id) {
                Some(state) => {
                    let (result_sender, result_receiver) = oneshot::channel();

                    match &mut state.typ {
                        TransfersServiceTransferStateType::Download(download) => {
                            download.reader_provider = Some(Arc::new(reader_provider));
                            download.downloadable =
                                Some(Arc::new(futures::lock::Mutex::new(downloadable)));
                            download.result_sender = Some(result_sender);
                        }
                        _ => {}
                    }

                    result_receiver
                }
                None => return Err(TransferError::Aborted),
            };

            mutations::create_transfer(
                state,
                notify,
                id,
                mutations::CreateTransferType::Download { name },
                size,
                is_persistent,
                is_retriable,
                is_openable,
            );

            Ok(result_receiver)
        })?;

        self.process_next();

        Ok(async { result_receiver.await.unwrap() }.boxed())
    }

    pub fn download_reader(self: Arc<Self>, reader: RepoFileReader) -> DownloadReaderResult {
        let id = self.get_next_id();

        let (abort_handle, _) = AbortHandle::new_pair();

        self.store.mutate(|state, notify, _, _| {
            self.state.write().unwrap().transfers.insert(
                id,
                TransfersServiceTransferState {
                    typ: TransfersServiceTransferStateType::DownloadReader,
                    abort_handle: Some(abort_handle.clone()),
                },
            );

            mutations::create_download_reader_transfer(
                state,
                notify,
                id,
                reader.name.clone(),
                reader.size,
                self.runtime.now_ms(),
            )
        });

        let reader = reader.wrap_reader(|reader| {
            let abort_reader = AbortReader::new(reader, abort_handle);

            let progress_reader =
                ProgressReader::new(abort_reader, self.clone().get_transfer_on_progress(id));

            let this = self.clone();

            Box::pin(OnEndReader::new(
                progress_reader,
                Box::new(move |_| {
                    this.store.mutate(|state, notify, _, _| {
                        this.state.write().unwrap().transfers.remove(&id);

                        mutations::cleanup_download_reader_transfer(state, notify, id);
                    });
                }),
            ))
        });

        (id, reader)
    }

    pub fn abort(self: Arc<Self>, id: u32) {
        let state = self.store.mutate(|state, notify, _, _| {
            mutations::abort(state, notify, id);

            self.state.write().unwrap().transfers.remove(&id)
        });

        if let Some(state) = state {
            self.handle_abort(state);
        }

        self.process_next();
    }

    pub fn abort_all(self: Arc<Self>) {
        let states: Vec<_> = self.store.mutate(|state, notify, _, _| {
            let ids = mutations::abort_all(state, notify);

            let mut state = self.state.write().unwrap();

            ids.iter()
                .filter_map(|id| state.transfers.remove(&id))
                .collect()
        });

        for state in states {
            self.handle_abort(state);
        }

        self.process_next();
    }

    fn handle_abort(&self, state: TransfersServiceTransferState) {
        match state.typ {
            TransfersServiceTransferStateType::Upload(mut upload) => {
                if let Some(sender) = upload.result_sender.take() {
                    let _ = sender.send(Err(TransferError::Aborted));
                }
            }
            TransfersServiceTransferStateType::Download(mut download) => {
                if let Some(sender) = download.result_sender.take() {
                    let _ = sender.send(Err(TransferError::Aborted));
                }
            }
            TransfersServiceTransferStateType::DownloadReader => {}
        }

        if let Some(handle) = state.abort_handle {
            handle.abort();
        }
    }

    pub fn retry(self: Arc<Self>, id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::retry(
                state,
                notify,
                id,
                RetryInitiator::User,
                self.runtime.now_ms(),
            );
        });

        self.process_next();
    }

    pub fn retry_all(self: Arc<Self>) {
        self.store.mutate(|state, notify, _, _| {
            mutations::retry_all(state, notify, self.runtime.now_ms());
        });

        self.process_next();
    }

    pub async fn open(self: Arc<Self>, id: u32) -> Result<(), TransferError> {
        let downloadable = {
            let state = self.state.read().unwrap();

            let state = state
                .transfers
                .get(&id)
                .ok_or(TransferError::TransferNotFound)?;

            (match &state.typ {
                TransfersServiceTransferStateType::Download(download) => {
                    download.downloadable.clone()
                }
                _ => None,
            })
            .ok_or(TransferError::NotOpenable)?
        };
        let downloadable = downloadable.lock().await;

        Ok(downloadable.open().await?)
    }

    fn process_next(self: Arc<Self>) {
        loop {
            let (id, abort_registration) = match self.store.mutate(|state, notify, _, _| {
                let id = mutations::next_transfer(state, notify, self.runtime.now_ms())?;

                let mut state = self.state.write().unwrap();

                let (abort_handle, abort_registration) = AbortHandle::new_pair();

                if let Some(state) = state.transfers.get_mut(&id) {
                    state.abort_handle = Some(abort_handle);
                }

                Some((id, abort_registration))
            }) {
                Some((id, abort_registration)) => (id, abort_registration),
                None => break,
            };

            self.runtime.spawn(Box::pin(
                self.clone().handle_transfer(id, abort_registration),
            ));
        }
    }

    async fn handle_transfer(self: Arc<Self>, id: u32, abort_registration: AbortRegistration) {
        let res = Abortable::new(self.clone().process_transfer(id), abort_registration).await;

        match res {
            Ok(Ok(send_result)) => {
                self.store.mutate(|state, notify, _, _| {
                    if mutations::transfer_done(state, notify, id) {
                        self.state.write().unwrap().transfers.remove(&id);
                    }
                });

                // we send the result after the store is updated
                send_result();
            }
            Ok(Err(err)) => {
                self.store.mutate(|state, notify, _, _| {
                    mutations::transfer_failed(
                        state,
                        notify,
                        id,
                        err.into(),
                        self.runtime.now_ms(),
                    );

                    if let Some(state) = self.state.write().unwrap().transfers.get_mut(&id) {
                        state.abort_handle = None;
                    }
                });
            }
            Err(Aborted) => {}
        }

        self.process_next();
    }

    async fn process_transfer(
        self: Arc<Self>,
        id: u32,
    ) -> Result<Box<dyn FnOnce()>, TransferError> {
        match self.store.with_state(|state| {
            selectors::select_transfer(state, id).map(|transfer| transfer.typ.clone())
        }) {
            Some(TransferType::Upload(upload_transfer)) => {
                self.process_upload_transfer(id, upload_transfer).await
            }
            Some(TransferType::Download(_)) => self.process_download_transfer(id).await,
            Some(TransferType::DownloadReader) => Ok(Box::new(|| {})), // unreachable
            None => return Err(TransferError::TransferNotFound),
        }
    }

    async fn process_upload_transfer(
        self: Arc<Self>,
        id: u32,
        upload_transfer: UploadTransfer,
    ) -> Result<Box<dyn FnOnce()>, TransferError> {
        if !self.store.with_state(|state| {
            repo_files_selectors::select_is_root_loaded(
                state,
                &upload_transfer.repo_id,
                &upload_transfer.parent_path,
            )
        }) {
            match self
                .repo_files_service
                .load_files(&upload_transfer.repo_id, &upload_transfer.parent_path)
                .await
            {
                Ok(()) => {}
                Err(LoadFilesError::RemoteError(err))
                    if err.is_api_error_code(ApiErrorCode::NotFound) => {}
                Err(err) => return Err(err.into()),
            }
        }

        let uploadable = self
            .state
            .read()
            .unwrap()
            .transfers
            .get(&id)
            .and_then(|state| match &state.typ {
                TransfersServiceTransferStateType::Upload(upload) => upload.uploadable.clone(),
                _ => None,
            })
            .ok_or(TransferError::TransferNotFound)?;

        let (reader, size) = uploadable.reader().await?;

        let name = self.store.mutate(|state, notify, _, _| {
            mutations::upload_transfer_processed(state, notify, id, size)
        })?;

        let size = match size {
            SizeInfo::Exact(size) => Some(size),
            _ => None,
        };

        let res = self
            .repo_files_service
            .clone()
            .upload_file_reader(
                &upload_transfer.repo_id,
                &upload_transfer.parent_path,
                &name,
                reader,
                size,
                RepoFilesUploadConflictResolution::Error,
                Some(self.clone().get_transfer_on_progress(id)),
            )
            .await?;

        let sender = self
            .state
            .write()
            .unwrap()
            .transfers
            .get_mut(&id)
            .and_then(|state| match &mut state.typ {
                TransfersServiceTransferStateType::Upload(upload) => upload.result_sender.take(),
                _ => None,
            });

        Ok(Box::new(move || {
            if let Some(sender) = sender {
                let _ = sender.send(Ok(res));
            }
        }))
    }

    async fn process_download_transfer(
        self: Arc<Self>,
        id: u32,
    ) -> Result<Box<dyn FnOnce()>, TransferError> {
        let (reader_provider, downloadable) = self
            .state
            .read()
            .unwrap()
            .transfers
            .get(&id)
            .and_then(|state| match &state.typ {
                TransfersServiceTransferStateType::Download(download) => {
                    match (
                        download.reader_provider.clone(),
                        download.downloadable.clone(),
                    ) {
                        (Some(reader_provider), Some(downloadable)) => {
                            Some((reader_provider, downloadable))
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .ok_or(TransferError::TransferNotFound)?;

        let mut downloadable = downloadable.lock().await;

        let reader = reader_provider.reader().await?;

        let progress_reader =
            ProgressReader::new(reader.reader, self.clone().get_transfer_on_progress(id));

        let name = reader.name.clone();
        let unique_name = reader.unique_name.clone();

        let (mut writer, name) = downloadable
            .writer(name, reader.size, reader.content_type, unique_name)
            .await?;

        self.store.mutate(|state, notify, _, _| {
            mutations::download_transfer_processed(state, notify, id, name, reader.size)
        })?;

        let copy_res = io::copy_buf(
            BufReader::with_capacity(1024 * 1024, progress_reader),
            &mut writer,
        )
        .await;

        match writer.close().await {
            Ok(_) => {}
            Err(err) => {
                let err: DownloadableError = err.into();

                let _ = downloadable.done(Err(err.clone())).await;

                return Err(err.into());
            }
        }

        drop(writer);

        match copy_res {
            Ok(_) => {
                downloadable
                    .done(Ok(DownloadableStatus::Downloaded))
                    .await?
            }
            Err(err) => {
                let err: DownloadableError = err.into();

                let _ = downloadable.done(Err(err.clone())).await;

                return Err(err.into());
            }
        }

        let sender = self
            .state
            .write()
            .unwrap()
            .transfers
            .get_mut(&id)
            .and_then(|state| match &mut state.typ {
                TransfersServiceTransferStateType::Download(download) => {
                    download.reader_provider = None;

                    download.result_sender.take()
                }
                _ => None,
            });

        Ok(Box::new(move || {
            if let Some(sender) = sender {
                let _ = sender.send(Ok(()));
            }
        }))
    }

    fn get_transfer_on_progress(self: Arc<Self>, id: u32) -> Box<dyn Fn(usize) + Send + Sync> {
        Box::new(move |n| {
            self.store.mutate(|state, notify, _, _| {
                mutations::transfer_progress(state, notify, id, n as i64, self.runtime.now_ms());
            });
        })
    }
}
