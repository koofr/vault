use std::sync::Arc;

use futures::{
    future::{self, BoxFuture},
    AsyncReadExt,
};

use crate::{
    eventstream::{self, service::MountSubscription},
    http::HttpError,
    remote::RemoteError,
    remote_files::errors::RemoteFilesErrors,
    repo_files::{errors::LoadFilesError, state::RepoFile, RepoFilesService},
    repo_files_read::{errors::GetFilesReaderError, state::RepoFileReader, RepoFilesReadService},
    repos::selectors as repos_selectors,
    store,
    utils::path_utils::normalize_path,
};

use super::{mutations, selectors, state::RepoFilesDetailsLocation};

pub struct RepoFilesDetailsService {
    repo_files_service: Arc<RepoFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    eventstream_service: Arc<eventstream::EventStreamService>,
    store: Arc<store::Store>,
}

impl RepoFilesDetailsService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        eventstream_service: Arc<eventstream::EventStreamService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            repo_files_service,
            repo_files_read_service,
            eventstream_service,
            store,
        }
    }

    pub fn create(
        self: Arc<Self>,
        repo_id: &str,
        path: &str,
    ) -> (u32, BoxFuture<'static, Result<(), LoadFilesError>>) {
        let location = self.clone().get_location(repo_id, path);

        let details_id = self.store.mutate(|state, notify| {
            notify(store::Event::RepoFilesDetails);

            mutations::create(state, location)
        });

        let load_self = self.clone();

        let load_future: BoxFuture<'static, Result<(), LoadFilesError>> = if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, details_id))
        {
            Box::pin(async move { load_self.load_file(details_id).await })
        } else {
            Box::pin(future::ready(Ok(())))
        };

        (details_id, load_future)
    }

    fn get_location(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<RepoFilesDetailsLocation, LoadFilesError> {
        normalize_path(path)
            .map(|path| {
                let eventstream_mount_subscription =
                    self.clone().get_eventstream_mount_subscription(repo_id);

                mutations::create_location(repo_id.to_owned(), path, eventstream_mount_subscription)
            })
            .map_err(|_| LoadFilesError::RemoteError(RemoteFilesErrors::invalid_path()))
    }

    fn get_eventstream_mount_subscription(&self, repo_id: &str) -> Option<Arc<MountSubscription>> {
        self.store
            .with_state(|state| {
                repos_selectors::select_repo(state, repo_id)
                    .map(|repo| (repo.mount_id.clone(), repo.path.clone()))
            })
            .ok()
            .map(|(mount_id, mount_path)| {
                self.eventstream_service
                    .clone()
                    .get_mount_subscription(&mount_id, &mount_path)
            })
    }

    pub fn destroy(&self, details_id: u32) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoFilesDetails);

            mutations::destroy(state, details_id);
        });
    }

    pub async fn load_file(&self, details_id: u32) -> Result<(), LoadFilesError> {
        if let Some((repo_id, path)) = self
            .store
            .with_state(|state| selectors::select_repo_id_path_owned(state, details_id))
        {
            let res = self.repo_files_service.load_files(&repo_id, &path).await;

            self.store.mutate(|state, notify| {
                notify(store::Event::RepoFilesDetails);

                mutations::loaded(state, details_id, &repo_id, &path, res.as_ref().err());
            });

            res?;
        }

        Ok(())
    }

    pub async fn load_content(self: Arc<Self>, details_id: u32) -> Result<(), GetFilesReaderError> {
        let file = self
            .store
            .mutate(|state, notify| -> Result<RepoFile, GetFilesReaderError> {
                notify(store::Event::RepoFilesDetails);

                let file = selectors::select_file(state, details_id)
                    .map(|file| file.clone())
                    .ok_or(GetFilesReaderError::FileNotFound)?;

                mutations::content_loading(state, details_id);

                Ok(file)
            })?;

        let repo_id = file.repo_id.clone();
        let path = file.path.decrypted_path()?.to_owned();

        let res = match self
            .repo_files_read_service
            .clone()
            .get_files_reader(&[file])
            .await
        {
            Ok(mut reader) => {
                let mut buf = Vec::new();

                match reader.reader.read_to_end(&mut buf).await {
                    Ok(_) => Ok(buf),
                    Err(err) => Err(GetFilesReaderError::RemoteError(RemoteError::HttpError(
                        HttpError::ResponseError(err.to_string()),
                    ))),
                }
            }
            Err(err) => Err(err),
        };

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoFilesDetails);

            mutations::content_loaded(state, details_id, repo_id, path, res);
        });

        res_err
    }

    pub async fn get_file_reader(
        self: Arc<Self>,
        details_id: u32,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let file = self
            .store
            .with_state(|state| selectors::select_file(state, details_id).map(|file| file.clone()))
            .ok_or(GetFilesReaderError::FileNotFound)?;

        self.repo_files_read_service
            .clone()
            .get_files_reader(&[file])
            .await
    }
}
