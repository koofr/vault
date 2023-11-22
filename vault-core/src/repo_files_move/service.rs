use std::sync::Arc;

use crate::{
    dir_pickers::{selectors as dir_pickers_selectors, state::DirPickerItemId},
    repo_files::{
        errors::{CreateDirError, MoveFileError, RepoFilesErrors},
        RepoFilesService,
    },
    repo_files_dir_pickers::RepoFilesDirPickersService,
    store,
    types::{EncryptedPath, RepoFileId, RepoId},
    utils::repo_encrypted_path_utils,
};

use super::{
    errors::{DirPickerClickError, ShowError},
    mutations, selectors,
    state::RepoFilesMoveMode,
};

pub struct RepoFilesMoveService {
    repo_files_service: Arc<RepoFilesService>,
    repo_files_dir_pickers_service: Arc<RepoFilesDirPickersService>,
    store: Arc<store::Store>,
}

impl RepoFilesMoveService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        repo_files_dir_pickers_service: Arc<RepoFilesDirPickersService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            repo_files_service,
            repo_files_dir_pickers_service,
            store,
        }
    }

    pub async fn move_file(
        &self,
        repo_id: RepoId,
        path: EncryptedPath,
        mode: RepoFilesMoveMode,
    ) -> Result<(), ShowError> {
        self.show(repo_id, vec![path], mode).await
    }

    pub async fn show(
        &self,
        repo_id: RepoId,
        src_paths: Vec<EncryptedPath>,
        mode: RepoFilesMoveMode,
    ) -> Result<(), ShowError> {
        let first_src_path = src_paths.get(0).ok_or(ShowError::FilesEmpty)?;
        let dest_path = repo_encrypted_path_utils::parent_path(first_src_path)
            .ok_or(RepoFilesErrors::move_root())?;

        let dir_picker_id = self.repo_files_dir_pickers_service.create(repo_id.clone());

        self.store.mutate(|state, notify, _, _| {
            mutations::show(
                state,
                notify,
                repo_id,
                src_paths,
                dest_path.clone(),
                mode,
                dir_picker_id,
            );
        });

        self.repo_files_dir_pickers_service
            .select_file(dir_picker_id, &dest_path)
            .await?;

        self.repo_files_dir_pickers_service
            .load(dir_picker_id)
            .await?;

        // try to select it again if the data was not loaded yet
        if self.store.with_state(|state| {
            dir_pickers_selectors::select_selected_id(state, dir_picker_id).is_none()
        }) {
            self.repo_files_dir_pickers_service
                .select_file(dir_picker_id, &dest_path)
                .await?;
        }

        Ok(())
    }

    pub fn set_dest_path(&self, dest_path: EncryptedPath) {
        self.store
            .mutate(|state, notify, _, _| mutations::set_dest_path(state, notify, dest_path))
    }

    pub async fn dir_picker_click(
        &self,
        item_id: &DirPickerItemId,
        is_arrow: bool,
    ) -> Result<(), DirPickerClickError> {
        let (dir_picker_id, dest_path) = self.store.with_state(|state| {
            selectors::select_dir_picker_click(state, &RepoFileId(item_id.0.to_owned()))
        })?;

        self.set_dest_path(dest_path);

        Ok(self
            .repo_files_dir_pickers_service
            .click(dir_picker_id, item_id, is_arrow)
            .await?)
    }

    pub async fn move_files(&self) -> Result<(), MoveFileError> {
        let info = self
            .store
            .mutate(|state, notify, _, _| mutations::move_files(state, notify))?;

        self.repo_files_dir_pickers_service
            .destroy(info.dir_picker_id);

        for src_path in info.src_paths {
            match info.mode {
                RepoFilesMoveMode::Copy => {
                    self.repo_files_service
                        .copy_file(&info.repo_id, &src_path, &info.dest_path)
                        .await?
                }
                RepoFilesMoveMode::Move => {
                    self.repo_files_service
                        .move_file(&info.repo_id, &src_path, &info.dest_path)
                        .await?
                }
            }
        }

        Ok(())
    }

    pub fn cancel(&self) {
        if let Some(dir_picker_id) = self
            .store
            .mutate(|state, notify, _, _| mutations::cancel(state, notify))
        {
            self.repo_files_dir_pickers_service.destroy(dir_picker_id);
        }
    }

    pub async fn create_dir(&self) -> Result<(), CreateDirError> {
        let (repo_id, parent_path, dir_picker_id) = self
            .store
            .with_state(|state| {
                state
                    .repo_files_move
                    .as_ref()
                    .map(|x| (x.repo_id.clone(), x.dest_path.clone(), x.dir_picker_id))
            })
            .ok_or_else(RepoFilesErrors::not_found)?;

        let (_, path) = self
            .repo_files_service
            .create_dir(&repo_id, &parent_path)
            .await?;

        self.set_dest_path(path.clone());

        self.repo_files_dir_pickers_service
            .select_file(dir_picker_id, &path)
            .await?;

        Ok(())
    }
}
