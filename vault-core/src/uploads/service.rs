use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, RwLock},
};

use futures::{
    channel::oneshot::{self, Receiver, Sender},
    future::Shared,
    FutureExt,
};

use crate::{
    repo_files::{
        self,
        errors::UploadFileReaderError,
        state::{RepoFilesUploadConflictResolution, RepoFilesUploadResult},
        RepoFilesService,
    },
    runtime, store,
};

use super::{errors::UploadError, mutations, selectors};

pub type Uploadable = Pin<Box<dyn repo_files::state::RepoFileUploadable + Send + Sync>>;

pub type UploadResult = Result<RepoFilesUploadResult, UploadError>;

pub struct UploadsService {
    repo_files_service: Arc<RepoFilesService>,
    store: Arc<store::Store>,
    runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
    uploadables: Arc<RwLock<HashMap<u32, Uploadable>>>,
    results: Arc<RwLock<HashMap<u32, Sender<UploadResult>>>>,
    abort_senders: Arc<RwLock<HashMap<u32, Sender<()>>>>,
    abort_receivers: Arc<RwLock<HashMap<u32, Shared<Receiver<()>>>>>,
}

impl UploadsService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        store: Arc<store::Store>,
        runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
    ) -> Self {
        Self {
            repo_files_service,
            store,
            runtime,
            uploadables: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            abort_senders: Arc::new(RwLock::new(HashMap::new())),
            abort_receivers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn now(&self) -> i64 {
        instant::now() as i64
    }

    pub async fn upload(
        self: Arc<Self>,
        repo_id: &str,
        parent_path: &str,
        name: &str,
        uploadable: Uploadable,
    ) -> UploadResult {
        let id = self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::get_next_id(state)
        });

        let size = uploadable.size();

        self.uploadables.write().unwrap().insert(id, uploadable);

        let (result_sender, result_receiver) = oneshot::channel();

        self.results.write().unwrap().insert(id, result_sender);

        let (abort_sender, abort_receiver) = oneshot::channel();

        self.abort_senders.write().unwrap().insert(id, abort_sender);
        self.abort_receivers
            .write()
            .unwrap()
            .insert(id, abort_receiver.shared());

        self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::file_upload_added(
                state,
                mutations::FileUploadAdded {
                    id,
                    repo_id: repo_id.to_owned(),
                    parent_path: parent_path.to_owned(),
                    name: name.to_owned(),
                    size,
                    is_persistent: false,
                },
                self.now(),
            );
        });

        self.process_next();

        result_receiver.await.unwrap()
    }

    pub fn abort_file(&self, id: u32) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::file_upload_abort(state, id);
        });

        self.abort_file_cleanup(id);
    }

    pub fn abort_file_cleanup(&self, id: u32) {
        self.uploadables.write().unwrap().remove(&id);

        if let Some(sender) = self.results.write().unwrap().remove(&id) {
            let _ = sender.send(Err(UploadError::Aborted));
        }

        if let Some(sender) = self.abort_senders.write().unwrap().remove(&id) {
            let _ = sender.send(());
        }

        self.abort_receivers.write().unwrap().remove(&id);
    }

    pub fn abort_all(&self) {
        let ids = self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::file_upload_abort_all(state)
        });

        for id in ids {
            self.abort_file_cleanup(id);
        }
    }

    pub fn retry_file(self: Arc<Self>, id: u32) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::file_upload_retry(state, id, self.now());
        });

        self.process_next();
    }

    pub fn retry_all(self: Arc<Self>) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::file_upload_retry_all(state, self.now());
        });

        self.process_next();
    }

    fn process_next(self: Arc<Self>) {
        if let Some(next_file_id) = self
            .store
            .with_state(|state| selectors::select_next_file(state).map(|file| file.id))
        {
            self.clone().upload_file(next_file_id);

            self.process_next();
        }
    }

    fn upload_file(self: Arc<Self>, id: u32) {
        self.store.mutate(|state, notify| {
            notify(store::Event::Uploads);

            mutations::file_upload_uploading(state, id, self.now());
        });

        let (repo_id, parent_path, autorename_name) = match self.store.with_state(|state| {
            selectors::select_file(state, id).map(|file| {
                (
                    file.repo_id.clone(),
                    file.parent_path.clone(),
                    file.autorename_name
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| file.name.clone()),
                )
            })
        }) {
            Some(file) => file,
            None => {
                return;
            }
        };

        let uploadable = match self.uploadables.write().unwrap().remove(&id) {
            Some(uploadable) => uploadable,
            None => {
                return;
            }
        };

        let abort = match self.abort_receivers.read().unwrap().get(&id) {
            Some(receiver) => Some(
                receiver
                    .clone()
                    .map(|res| res.map_err(|_| ()))
                    .boxed()
                    .shared(),
            ),
            None => None,
        };

        let size = uploadable.size();
        let reader = uploadable.reader();

        let upload_future_self = self.clone();

        let upload_future = async move {
            let progress_self = upload_future_self.clone();

            match upload_future_self
                .repo_files_service
                .clone()
                .upload_file_reader(
                    &repo_id,
                    &parent_path,
                    &autorename_name,
                    reader,
                    size,
                    RepoFilesUploadConflictResolution::Error,
                    Some(Box::new(move |n| {
                        progress_self.store.mutate(|state, notify| {
                            notify(store::Event::Uploads);

                            mutations::file_upload_progress(state, id, n as i64);
                        });
                    })),
                    abort,
                )
                .await
            {
                Ok(res) => {
                    upload_future_self.store.mutate(|state, notify| {
                        notify(store::Event::Uploads);

                        mutations::file_upload_done(state, id);
                    });

                    if let Some(sender) = upload_future_self.results.write().unwrap().remove(&id) {
                        let _ = sender.send(Ok(res));
                    }

                    upload_future_self.process_next();
                }
                Err(err) => {
                    let err = match err {
                        UploadFileReaderError::RepoNotFound(err) => UploadError::RepoNotFound(err),
                        UploadFileReaderError::RepoLocked(err) => UploadError::RepoLocked(err),
                        UploadFileReaderError::DecryptFilenameError(err) => {
                            UploadError::DecryptFilenameError(err)
                        }
                        UploadFileReaderError::RemoteError(err) => UploadError::RemoteError(err),
                    };

                    upload_future_self
                        .uploadables
                        .write()
                        .unwrap()
                        .insert(id, uploadable);

                    upload_future_self.store.mutate(|state, notify| {
                        notify(store::Event::Uploads);

                        mutations::file_upload_failed(state, id, err);
                    });

                    if upload_future_self
                        .store
                        .with_state(|state| selectors::select_can_auto_retry(state, id))
                    {
                        upload_future_self.retry_file(id);
                    } else {
                        upload_future_self.process_next();
                    }
                }
            };
        };

        self.runtime.spawn(Box::pin(upload_future));
    }
}
