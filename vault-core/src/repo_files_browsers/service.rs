use std::sync::Arc;

use futures::future::{self, BoxFuture};

use crate::{
    eventstream::{self, service::MountSubscription},
    remote_files::errors::RemoteFilesErrors,
    repo_files::{
        errors::{
            CreateDirError, CreateFileError, DeleteFileError, LoadFilesError, RepoFilesErrors,
        },
        state::{RepoFile, RepoFilesSortField},
        RepoFilesService,
    },
    repo_files_move::{errors::ShowError, state::RepoFilesMoveMode, RepoFilesMoveService},
    repo_files_read::{
        errors::GetFilesReaderError, state::RepoFileReaderProvider, RepoFilesReadService,
    },
    repos::selectors as repos_selectors,
    sort::state::SortDirection,
    store,
    utils::path_utils::normalize_path,
};

use super::{
    mutations, selectors,
    state::{RepoFilesBrowserLocation, RepoFilesBrowserOptions},
};

pub struct RepoFilesBrowsersService {
    repo_files_service: Arc<RepoFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    repo_files_move_service: Arc<RepoFilesMoveService>,
    eventstream_service: Arc<eventstream::EventStreamService>,
    store: Arc<store::Store>,
    repo_files_mutation_subscription_id: u32,
}

impl RepoFilesBrowsersService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        repo_files_move_service: Arc<RepoFilesMoveService>,
        eventstream_service: Arc<eventstream::EventStreamService>,
        store: Arc<store::Store>,
    ) -> Self {
        let repo_files_mutation_subscription_id = store.get_next_id();

        store.mutation_on(
            repo_files_mutation_subscription_id,
            &[store::MutationEvent::RepoFiles],
            Box::new(|state, notify, _, _| {
                mutations::handle_repo_files_mutation(state, notify);
            }),
        );

        Self {
            repo_files_service,
            repo_files_read_service,
            repo_files_move_service,
            eventstream_service,
            store: store.clone(),
            repo_files_mutation_subscription_id,
        }
    }

    pub fn create(
        self: Arc<Self>,
        repo_id: &str,
        path: &str,
        options: RepoFilesBrowserOptions,
    ) -> (u32, BoxFuture<'static, Result<(), LoadFilesError>>) {
        let location = self.clone().get_location(repo_id, path);

        let browser_id = self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::create(state, options, location)
        });

        let load_self = self.clone();

        let load_future: BoxFuture<'static, Result<(), LoadFilesError>> = if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, browser_id))
        {
            Box::pin(async move { load_self.load_files(browser_id).await })
        } else {
            Box::pin(future::ready(Ok(())))
        };

        (browser_id, load_future)
    }

    fn get_location(
        &self,
        repo_id: &str,
        path: &str,
    ) -> Result<RepoFilesBrowserLocation, LoadFilesError> {
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

    pub fn destroy(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::destroy(state, browser_id);
        });
    }

    pub async fn set_location(
        &self,
        browser_id: u32,
        repo_id: &str,
        path: &str,
    ) -> Result<(), LoadFilesError> {
        let location = self.clone().get_location(repo_id, path);

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

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

    pub async fn load_files(&self, browser_id: u32) -> Result<(), LoadFilesError> {
        if let Some((repo_id, path)) = self
            .store
            .with_state(|state| selectors::select_repo_id_path_owned(state, browser_id))
        {
            let res = self.repo_files_service.load_files(&repo_id, &path).await;

            self.store.mutate(|state, notify, _, _| {
                notify(store::Event::RepoFilesBrowsers);

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
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::select_file(state, browser_id, file_id, extend, range, force);
        });
    }

    pub fn select_all(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::select_all(state, browser_id);
        });
    }

    pub fn clear_selection(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::clear_selection(state, browser_id);
        });
    }

    pub fn set_selection(&self, browser_id: u32, selection: Vec<String>) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::set_selection(state, browser_id, selection);
        });
    }

    pub fn sort_by(
        &self,
        browser_id: u32,
        field: RepoFilesSortField,
        direction: Option<SortDirection>,
    ) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesBrowsers);

            mutations::sort_by(state, browser_id, field, direction);
        });
    }

    pub fn get_selected_reader(
        self: Arc<Self>,
        browser_id: u32,
    ) -> Result<RepoFileReaderProvider, GetFilesReaderError> {
        let files: Vec<RepoFile> = self.store.with_state(|state| {
            selectors::select_selected_files(state, browser_id)
                .into_iter()
                .map(|file| file.clone())
                .collect()
        });

        if files.is_empty() {
            return Err(GetFilesReaderError::FilesEmpty);
        }

        self.repo_files_read_service.clone().get_files_reader(files)
    }

    pub async fn create_dir(&self, browser_id: u32) -> Result<(String, String), CreateDirError> {
        let (repo_id, parent_path) =
            self.store
                .with_state::<_, Result<_, CreateDirError>>(|state| {
                    let root_file = selectors::select_root_file(state, browser_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    let root_path = root_file.decrypted_path()?;

                    Ok((root_file.repo_id.clone(), root_path.to_owned()))
                })?;

        Ok(self
            .repo_files_service
            .create_dir(&repo_id, &parent_path)
            .await?)
    }

    pub async fn create_file(
        &self,
        browser_id: u32,
        name: &str,
    ) -> Result<(String, String), CreateFileError> {
        let (repo_id, parent_path) =
            self.store
                .with_state::<_, Result<_, CreateFileError>>(|state| {
                    let root_file = selectors::select_root_file(state, browser_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    let root_path = root_file.decrypted_path()?;

                    Ok((root_file.repo_id.clone(), root_path.to_owned()))
                })?;

        Ok(self
            .repo_files_service
            .clone()
            .create_file(&repo_id, &parent_path, name)
            .await?)
    }

    pub async fn delete_selected(&self, browser_id: u32) -> Result<(), DeleteFileError> {
        let files = self.store.with_state(|state| {
            selectors::select_selected_files(state, browser_id)
                .into_iter()
                .filter_map(|file| {
                    file.path
                        .decrypted_path()
                        .ok()
                        .map(|path| (file.repo_id.clone(), path.to_owned()))
                })
                .collect::<Vec<_>>()
        });

        self.repo_files_service.delete_files(&files, None).await?;

        Ok(())
    }

    pub async fn move_selected(
        &self,
        browser_id: u32,
        mode: RepoFilesMoveMode,
    ) -> Result<(), ShowError> {
        let (repo_id, paths) = match self.store.with_state(|state| {
            let repo_id = selectors::select_repo_id(state, browser_id)?.to_owned();
            let paths = selectors::select_selected_paths(state, browser_id);

            if paths.is_empty() {
                return None;
            }

            Some((repo_id, paths))
        }) {
            Some(x) => x,
            None => return Ok(()),
        };

        self.repo_files_move_service
            .show(repo_id, paths, mode)
            .await
    }
}

impl Drop for RepoFilesBrowsersService {
    fn drop(&mut self) {
        self.store
            .mutation_remove_listener(self.repo_files_mutation_subscription_id)
    }
}
