use std::sync::Arc;

use futures::{
    future::{self, BoxFuture},
    FutureExt,
};

use crate::{
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
    repos::ReposService,
    runtime::runtime,
    sort::state::SortDirection,
    store,
    types::{DecryptedName, EncryptedPath, RepoFileId, RepoId},
};

use super::{mutations, selectors, state::RepoFilesBrowserOptions};

pub struct RepoFilesBrowsersService {
    repos_service: Arc<ReposService>,
    repo_files_service: Arc<RepoFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    repo_files_move_service: Arc<RepoFilesMoveService>,
    store: Arc<store::Store>,
    repos_subscription_id: u32,
    repo_files_mutation_subscription_id: u32,
}

impl RepoFilesBrowsersService {
    pub fn new(
        repos_service: Arc<ReposService>,
        repo_files_service: Arc<RepoFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        repo_files_move_service: Arc<RepoFilesMoveService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Self {
        let repos_subscription_id = store.get_next_id();
        let repos_subscription_repos_service = repos_service.clone();
        let repos_subscription_repo_files_service = repo_files_service.clone();
        let repos_subscription_store = store.clone();
        let repos_subscription_runtime = runtime.clone();

        store.on(
            repos_subscription_id,
            &[store::Event::Repos],
            Box::new(move |mutation_state, add_side_effect| {
                if !mutation_state.repos.unlocked_repos.is_empty() {
                    for browser_id in repos_subscription_store.with_state(|state| {
                        selectors::select_unlocked_browsers(state, mutation_state)
                    }) {
                        let repos_service = repos_subscription_repos_service.clone();
                        let repo_files_service = repos_subscription_repo_files_service.clone();
                        let store = repos_subscription_store.clone();
                        let runtime = repos_subscription_runtime.clone();

                        add_side_effect(Box::new(move || {
                            // load errors are displayed inside browser
                            runtime.spawn(
                                Self::load_files_inner(
                                    repos_service.clone(),
                                    repo_files_service.clone(),
                                    store.clone(),
                                    browser_id,
                                )
                                .map(|_| ())
                                .boxed(),
                            )
                        }))
                    }
                }
            }),
        );

        let repo_files_mutation_subscription_id = store.get_next_id();
        let repo_files_mutation_repos_service = repos_service.clone();

        store.mutation_on(
            repo_files_mutation_subscription_id,
            &[store::MutationEvent::RepoFiles],
            Box::new(move |state, notify, _, _| {
                mutations::handle_repo_files_mutation(
                    state,
                    notify,
                    &repo_files_mutation_repos_service.get_ciphers(),
                );
            }),
        );

        Self {
            repos_service,
            repo_files_service,
            repo_files_read_service,
            repo_files_move_service,
            store,
            repos_subscription_id,
            repo_files_mutation_subscription_id,
        }
    }

    pub fn create(
        self: Arc<Self>,
        repo_id: RepoId,
        path: &EncryptedPath,
        options: RepoFilesBrowserOptions,
    ) -> (u32, BoxFuture<'static, Result<(), LoadFilesError>>) {
        let cipher = self.repos_service.get_cipher(&repo_id).ok();

        let browser_id = self.store.mutate(|state, notify, mutation_state, _| {
            mutations::create(
                state,
                notify,
                mutation_state,
                options,
                repo_id,
                path,
                cipher.as_deref(),
            )
        });

        let load_future: BoxFuture<'static, Result<(), LoadFilesError>> = if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, browser_id))
        {
            Self::load_files_inner(
                self.repos_service.clone(),
                self.repo_files_service.clone(),
                self.store.clone(),
                browser_id,
            )
            .boxed()
        } else {
            Box::pin(future::ready(Ok(())))
        };

        (browser_id, load_future)
    }

    pub fn destroy(&self, browser_id: u32) {
        self.store.mutate(|state, notify, mutation_state, _| {
            mutations::destroy(state, notify, mutation_state, browser_id);
        });
    }

    pub async fn load_files(&self, browser_id: u32) -> Result<(), LoadFilesError> {
        Self::load_files_inner(
            self.repos_service.clone(),
            self.repo_files_service.clone(),
            self.store.clone(),
            browser_id,
        )
        .await
    }

    pub async fn load_files_inner(
        repos_service: Arc<ReposService>,
        repo_files_service: Arc<RepoFilesService>,
        store: Arc<store::Store>,
        browser_id: u32,
    ) -> Result<(), LoadFilesError> {
        if let Some((repo_id, path)) =
            store.with_state(|state| selectors::select_repo_id_path_owned(state, browser_id))
        {
            store.mutate(|state, notify, _, _| {
                mutations::loading(state, notify, browser_id);
            });

            let cipher = repos_service.get_cipher(&repo_id).ok();

            let res = repo_files_service.load_files(&repo_id, &path).await;

            store.mutate(|state, notify, _, _| {
                mutations::loaded(
                    state,
                    notify,
                    browser_id,
                    &repo_id,
                    &path,
                    res.as_ref().err(),
                    cipher.as_deref(),
                );
            });

            res?;
        }

        Ok(())
    }

    pub fn select_file(
        &self,
        browser_id: u32,
        file_id: RepoFileId,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.store.mutate(|state, notify, _, _| {
            mutations::select_file(state, notify, browser_id, file_id, extend, range, force);
        });
    }

    pub fn select_all(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::select_all(state, notify, browser_id);
        });
    }

    pub fn clear_selection(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::clear_selection(state, notify, browser_id);
        });
    }

    pub fn set_selection(&self, browser_id: u32, selection: Vec<RepoFileId>) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_selection(state, notify, browser_id, selection);
        });
    }

    pub fn sort_by(
        &self,
        browser_id: u32,
        field: RepoFilesSortField,
        direction: Option<SortDirection>,
    ) {
        self.store.mutate(|state, notify, _, _| {
            let cipher = selectors::select_repo_id(state, browser_id)
                .and_then(|repo_id| self.repos_service.get_cipher(&repo_id).ok());

            mutations::sort_by(
                state,
                notify,
                browser_id,
                field,
                direction,
                cipher.as_deref(),
            );
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

    pub async fn create_dir(
        &self,
        browser_id: u32,
    ) -> Result<(DecryptedName, EncryptedPath), CreateDirError> {
        let (repo_id, parent_path) =
            self.store
                .with_state::<_, Result<_, CreateDirError>>(|state| {
                    let root_file = selectors::select_root_file(state, browser_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    Ok((root_file.repo_id.clone(), root_file.encrypted_path.clone()))
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
    ) -> Result<(DecryptedName, EncryptedPath), CreateFileError> {
        let (repo_id, parent_path) =
            self.store
                .with_state::<_, Result<_, CreateFileError>>(|state| {
                    let root_file = selectors::select_root_file(state, browser_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    Ok((root_file.repo_id.clone(), root_file.encrypted_path.clone()))
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
                .map(|file| (file.repo_id.clone(), file.encrypted_path.clone()))
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
        self.store.remove_listener(self.repos_subscription_id);
        self.store
            .mutation_remove_listener(self.repo_files_mutation_subscription_id);
    }
}
