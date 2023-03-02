use std::sync::Arc;

use futures::future::{self, BoxFuture};

use crate::{
    eventstream::{self, service::MountSubscription},
    remote_files::errors::RemoteFilesErrors,
    repo_files::{
        errors::{
            self as repo_files_errors, CreateDirError, DeleteFileError, GetFileReaderError,
            RepoFilesErrors,
        },
        state::{RepoFilePath, RepoFileReader},
        RepoFilesService,
    },
    repos::selectors as repos_selectors,
    store,
    utils::path_utils::normalize_path,
};

use super::{mutations, selectors, state::RepoFilesBrowserLocation};

pub struct RepoFilesBrowsersService {
    repo_files_service: Arc<RepoFilesService>,
    eventstream_service: Arc<eventstream::EventStreamService>,
    store: Arc<store::Store>,
}

impl RepoFilesBrowsersService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        eventstream_service: Arc<eventstream::EventStreamService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            repo_files_service,
            eventstream_service,
            store,
        }
    }

    pub fn create(
        self: Arc<Self>,
        repo_id: &str,
        path: &str,
    ) -> (
        u32,
        BoxFuture<'static, Result<(), repo_files_errors::LoadFilesError>>,
    ) {
        let location = self.clone().get_location(repo_id, path);

        let repo_files_subscription_id = self.store.get_next_id();

        let browser_id = self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
            mutations::create(state, location, repo_files_subscription_id)
        });

        let load_self = self.clone();

        let load_future: BoxFuture<'static, Result<(), repo_files_errors::LoadFilesError>> = if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, browser_id))
        {
            Box::pin(async move { load_self.load_files(browser_id).await })
        } else {
            Box::pin(future::ready(Ok(())))
        };

        let update_files_self = self.clone();

        self.store.on(
            repo_files_subscription_id,
            &[store::Event::RepoFiles],
            Box::new(move || {
                update_files_self.update_files(browser_id);
            }),
        );

        (browser_id, load_future)
    }

    fn get_location(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<RepoFilesBrowserLocation, repo_files_errors::LoadFilesError> {
        normalize_path(path)
            .map(|path| {
                let eventstream_mount_subscription =
                    self.clone().get_eventstream_mount_subscription(repo_id);

                RepoFilesBrowserLocation {
                    repo_id: repo_id.to_owned(),
                    path,
                    eventstream_mount_subscription,
                }
            })
            .map_err(|_| {
                repo_files_errors::LoadFilesError::RemoteError(RemoteFilesErrors::invalid_path())
            })
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

    fn update_files(&self, browser_id: u32) {
        if self
            .store
            .mutate_state(|state| mutations::update_files(state, browser_id))
        {
            self.store.notify(store::Event::RepoFilesBrowsers);
        }
    }

    pub fn destroy(&self, browser_id: u32) {
        let repo_files_subscription_id =
            self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
                mutations::destroy(state, browser_id)
            });

        if let Some(repo_files_subscription_id) = repo_files_subscription_id {
            self.store.remove_listener(repo_files_subscription_id);
        }
    }

    pub async fn set_location(
        &self,
        browser_id: u32,
        repo_id: &str,
        path: &str,
    ) -> Result<(), repo_files_errors::LoadFilesError> {
        let location = self.clone().get_location(repo_id, path);

        self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
            mutations::set_location(state, browser_id, location);
        });

        if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, browser_id))
        {
            self.load_files(browser_id).await?;
        }

        Ok(())
    }

    pub async fn load_files(
        &self,
        browser_id: u32,
    ) -> Result<(), repo_files_errors::LoadFilesError> {
        if let Some((repo_id, path)) = self
            .store
            .with_state(|state| selectors::select_repo_id_path_owned(state, browser_id))
        {
            let res = self.repo_files_service.load_files(&repo_id, &path).await;

            self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
                mutations::loaded(state, browser_id, &repo_id, &path, res.as_ref().err());
            });

            res?;
        }

        Ok(())
    }

    pub fn select_file(
        &self,
        browser_id: u32,
        file_id: &str,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
            mutations::select_file(state, browser_id, file_id, extend, range, force);
        });
    }

    pub fn toggle_select_all(&self, browser_id: u32) {
        self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
            mutations::toggle_select_all(state, browser_id);
        });
    }

    pub fn clear_selection(&self, browser_id: u32) {
        self.store.mutate(store::Event::RepoFilesBrowsers, |state| {
            mutations::clear_selection(state, browser_id);
        });
    }

    pub async fn get_selected_stream(
        &self,
        browser_id: u32,
    ) -> Result<RepoFileReader, GetFileReaderError> {
        let file_id = match self.store.with_state(|state| {
            selectors::select_info(state, browser_id).and_then(|info| {
                info.selected_file
                    .filter(|_| info.can_download_selected)
                    .map(|file| file.id.clone())
            })
        }) {
            Some(file_id) => file_id,
            None => {
                return Err(GetFileReaderError::FileNotFound);
            }
        };

        self.repo_files_service.get_file_reader(&file_id).await
    }

    pub fn check_create_dir(&self, browser_id: u32, name: &str) -> Result<(), CreateDirError> {
        self.store
            .with_state(|state| selectors::select_check_create_dir(state, browser_id, name))
    }

    pub async fn create_dir(&self, browser_id: u32, name: &str) -> Result<(), CreateDirError> {
        self.check_create_dir(browser_id, name)?;

        let (repo_id, parent_path) =
            self.store
                .with_state::<_, Result<_, CreateDirError>>(|state| {
                    let root_file = selectors::select_root_file(state, browser_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    let root_path = root_file.decrypted_path()?;

                    Ok((root_file.repo_id.clone(), root_path.to_owned()))
                })?;

        self.repo_files_service
            .create_dir(&repo_id, &parent_path, name)
            .await
    }

    pub async fn delete_selected(&self, browser_id: u32) -> Result<(), DeleteFileError> {
        for (repo_id, path) in self.store.with_state(|state| {
            selectors::select_selected_files(state, browser_id)
                .into_iter()
                .map(|file| (file.repo_id.clone(), file.path.clone()))
                .collect::<Vec<(String, RepoFilePath)>>()
        }) {
            if let Ok(path) = path.decrypted_path() {
                self.repo_files_service.delete_file(&repo_id, path).await?;
            }
        }

        Ok(())
    }
}
