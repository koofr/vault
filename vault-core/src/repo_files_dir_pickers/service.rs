use std::sync::Arc;

use futures::future::BoxFuture;

use crate::dir_pickers;
use crate::dir_pickers::state::DirPickerItem;
use crate::repo_files::errors::{CreateDirError, LoadFilesError, RepoFilesErrors};
use crate::repo_files::state::RepoFilePath;
use crate::repo_files::{selectors as repo_files_selectors, RepoFilesService};
use crate::store;
use crate::utils::path_utils;

use super::selectors;
use super::state::Options;

pub struct RepoFilesDirPickersService {
    repo_files_service: Arc<RepoFilesService>,
    store: Arc<store::Store>,
    helper: Arc<dir_pickers::DirPickersHelper<LoadFilesError>>,
}

impl RepoFilesDirPickersService {
    pub fn new(repo_files_service: Arc<RepoFilesService>, store: Arc<store::Store>) -> Self {
        let on_expand_repo_files_service = repo_files_service.clone();
        let on_expand_store = store.clone();

        let helper = Arc::new(dir_pickers::DirPickersHelper::<LoadFilesError>::new(
            store.clone(),
            Box::new(selectors::select_items),
            Box::new(move |_, item| {
                let on_expand_repo_files_service = on_expand_repo_files_service.clone();
                let on_expand_store = on_expand_store.clone();

                Box::pin(async move {
                    on_expand(item, on_expand_repo_files_service, on_expand_store).await
                })
            }),
        ));

        Self {
            repo_files_service,
            store,
            helper,
        }
    }

    pub fn create(&self, repo_id: &str) -> u32 {
        self.helper.clone().create(
            &[store::Event::RepoFiles],
            Options {
                repo_id: repo_id.to_owned(),
            },
        )
    }

    pub fn destroy(&self, picker_id: u32) {
        self.helper.destroy(picker_id);
    }

    fn get_repo_id(&self, picker_id: u32) -> Option<String> {
        self.store
            .with_state(|state| selectors::select_repo_id(state, picker_id))
    }

    pub async fn load(&self, picker_id: u32) -> Result<(), LoadFilesError> {
        if let Some(repo_id) = self.get_repo_id(picker_id) {
            self.repo_files_service.load_files(&repo_id, "/").await?;
        }

        Ok(())
    }

    pub async fn click(
        &self,
        picker_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) -> Result<(), LoadFilesError> {
        self.helper.click(picker_id, item_id, is_arrow).await
    }

    pub async fn select_file(&self, picker_id: u32, path: &str) -> Result<(), LoadFilesError> {
        let repo_id = match self.get_repo_id(picker_id) {
            Some(repo_id) => repo_id,
            None => return Ok(()),
        };

        let paths_chain = path_utils::paths_chain(&path);

        let mut expand_futures = Vec::<BoxFuture<Result<(), LoadFilesError>>>::new();

        for (idx, path) in paths_chain.iter().enumerate() {
            let item_id = repo_files_selectors::get_file_id(&repo_id, &path);

            if idx == paths_chain.len() - 1 {
                self.helper.set_selected(picker_id, &item_id);
            }

            expand_futures.push(self.helper.expand(picker_id, &item_id, false));
        }

        for expand_future in expand_futures {
            expand_future.await?;
        }

        Ok(())
    }

    pub async fn create_dir(&self, picker_id: u32, name: &str) -> Result<(), CreateDirError> {
        let (repo_id, parent_path) =
            self.store
                .with_state::<_, Result<_, CreateDirError>>(|state| {
                    selectors::select_check_create_dir(state, picker_id, name)?;

                    let parent_file = selectors::select_selected_file(state, picker_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    let parent_path = parent_file.decrypted_path()?;

                    Ok((parent_file.repo_id.clone(), parent_path.to_owned()))
                })?;

        self.repo_files_service
            .create_dir(&repo_id, &parent_path, name)
            .await?;

        let new_path = path_utils::join_path_name(&parent_path, name);

        self.select_file(picker_id, &new_path)
            .await
            .map_err(|e| match e {
                LoadFilesError::RepoNotFound(err) => CreateDirError::RepoNotFound(err),
                LoadFilesError::RepoLocked(err) => CreateDirError::RepoLocked(err),
                LoadFilesError::RemoteError(err) => CreateDirError::RemoteError(err),
            })?;

        Ok(())
    }
}

pub async fn on_expand(
    item: DirPickerItem,
    repo_files_service: Arc<RepoFilesService>,
    store: Arc<store::Store>,
) -> Result<(), LoadFilesError> {
    if let Some((repo_id, path)) = store.with_state(|state| {
        item.file_id
            .and_then(|file_id| repo_files_selectors::select_file(state, &file_id))
            .map(|file| (file.repo_id.clone(), file.path.clone()))
    }) {
        if let RepoFilePath::Decrypted { path } = path {
            repo_files_service.load_files(&repo_id, &path).await?;
        }
    }

    Ok(())
}
