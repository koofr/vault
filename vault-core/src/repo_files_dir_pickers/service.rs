use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{
    dir_pickers,
    dir_pickers::state::{DirPickerItem, DirPickerItemId},
    repo_files::{
        errors::LoadFilesError, selectors as repo_files_selectors, state::RepoFilePath,
        RepoFilesService,
    },
    store,
    types::{DecryptedPath, RepoFileId, RepoId, DECRYPTED_PATH_ROOT},
    utils::repo_path_utils,
};

use super::{selectors, state::Options};

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

    pub fn create(&self, repo_id: RepoId) -> u32 {
        self.helper
            .clone()
            .create(&[store::Event::RepoFiles], Options { repo_id })
    }

    pub fn destroy(&self, picker_id: u32) {
        self.helper.destroy(picker_id);
    }

    fn get_repo_id(&self, picker_id: u32) -> Option<RepoId> {
        self.store
            .with_state(|state| selectors::select_repo_id(state, picker_id))
    }

    pub async fn load(&self, picker_id: u32) -> Result<(), LoadFilesError> {
        if let Some(repo_id) = self.get_repo_id(picker_id) {
            self.repo_files_service
                .load_files(&repo_id, &DECRYPTED_PATH_ROOT)
                .await?;
        }

        Ok(())
    }

    pub async fn click(
        &self,
        picker_id: u32,
        item_id: &DirPickerItemId,
        is_arrow: bool,
    ) -> Result<(), LoadFilesError> {
        self.helper.click(picker_id, item_id, is_arrow).await
    }

    pub async fn select_file(
        &self,
        picker_id: u32,
        path: &DecryptedPath,
    ) -> Result<(), LoadFilesError> {
        let repo_id = match self.get_repo_id(picker_id) {
            Some(repo_id) => repo_id,
            None => return Ok(()),
        };

        let paths_chain = repo_path_utils::paths_chain(&path);

        let mut expand_futures = Vec::<BoxFuture<Result<(), LoadFilesError>>>::new();

        for (idx, path) in paths_chain.iter().enumerate() {
            let item_id = DirPickerItemId(repo_files_selectors::get_file_id(&repo_id, &path).0);

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
}

pub async fn on_expand(
    item: DirPickerItem,
    repo_files_service: Arc<RepoFilesService>,
    store: Arc<store::Store>,
) -> Result<(), LoadFilesError> {
    if let Some((repo_id, path)) = store.with_state(|state| {
        item.file_id
            .and_then(|file_id| {
                repo_files_selectors::select_file(state, &RepoFileId(file_id.0.clone()))
            })
            .map(|file| (file.repo_id.clone(), file.path.clone()))
    }) {
        if let RepoFilePath::Decrypted { path } = path {
            repo_files_service.load_files(&repo_id, &path).await?;
        }
    }

    Ok(())
}
