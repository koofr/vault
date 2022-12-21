use std::sync::Arc;

use crate::{
    dir_pickers::selectors as dir_pickers_selectors,
    repo_files::{
        errors::{CreateDirError, LoadFilesError, MoveFileError, RepoFilesErrors},
        selectors as repo_files_selectors, RepoFilesService,
    },
    repo_files_browsers::selectors as repo_files_browsers_selectors,
    repo_files_dir_pickers::RepoFilesDirPickersService,
    store,
    utils::path_utils,
};

use super::{
    selectors,
    state::{RepoFilesMoveMode, RepoFilesMoveState},
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

    pub async fn show(
        &self,
        browser_id: u32,
        mode: RepoFilesMoveMode,
    ) -> Result<(), LoadFilesError> {
        let repo_id = match self.store.with_state(|state| {
            repo_files_browsers_selectors::select_repo_id(state, browser_id).map(str::to_string)
        }) {
            Some(repo_id) => repo_id,
            None => {
                return Ok(());
            }
        };
        let src_file_ids = self.store.with_state(|state| {
            repo_files_browsers_selectors::select_selected_file_ids(state, browser_id)
                .into_iter()
                .map(|id| id.to_owned())
                .collect::<Vec<String>>()
        });
        if src_file_ids.is_empty() {
            return Ok(());
        }

        let first_file_path = self.store.with_state(|state| {
            repo_files_selectors::select_file(state, &src_file_ids.get(0).unwrap())
                .and_then(|file| file.decrypted_path().ok().map(str::to_string))
        });

        let dir_picker_id = self.repo_files_dir_pickers_service.create(&repo_id);

        self.store.mutate(store::Event::RepoFilesMove, |state| {
            state.repo_files_move = Some(RepoFilesMoveState {
                repo_id: repo_id.to_owned(),
                src_file_ids,
                mode,
                dir_picker_id,
            });
        });

        if let Some(first_file_path) = &first_file_path {
            if let Some(parent_path) = path_utils::parent_path(&first_file_path) {
                self.repo_files_dir_pickers_service
                    .select_file(dir_picker_id, parent_path)
                    .await?;
            }
        }

        self.repo_files_dir_pickers_service
            .load(dir_picker_id)
            .await?;

        // try to select it again if the data was not loaded yet
        if self.store.with_state(|state| {
            dir_pickers_selectors::select_selected_id(state, dir_picker_id).is_none()
        }) {
            if let Some(first_file_path) = &first_file_path {
                if let Some(parent_path) = path_utils::parent_path(&first_file_path) {
                    self.repo_files_dir_pickers_service
                        .select_file(dir_picker_id, parent_path)
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn move_files(&self) -> Result<(), MoveFileError> {
        let RepoFilesMoveState {
            repo_id,
            src_file_ids,
            mode,
            dir_picker_id,
        } = self
            .store
            .with_state::<_, Result<_, MoveFileError>>(|state| {
                selectors::select_check_move(state)?;

                Ok(state
                    .repo_files_move
                    .clone()
                    .ok_or_else(RepoFilesErrors::not_found)?)
            })?;

        let dest_file_id = self.store.with_state(|state| {
            dir_pickers_selectors::select_selected_file_id(state, dir_picker_id)
                .map(str::to_string)
                .ok_or_else(RepoFilesErrors::not_found)
        })?;

        let dest_parent_path = self
            .store
            .with_state::<_, Result<_, MoveFileError>>(|state| {
                let file = repo_files_selectors::select_file(state, &dest_file_id)
                    .ok_or_else(RepoFilesErrors::not_found)?;

                Ok(file.decrypted_path()?.to_owned())
            })?;

        self.cancel();

        for src_file_id in src_file_ids {
            let src_path = self
                .store
                .with_state::<_, Result<_, MoveFileError>>(|state| {
                    let file = repo_files_selectors::select_file(state, &src_file_id)
                        .ok_or_else(RepoFilesErrors::not_found)?;

                    Ok(file.decrypted_path()?.to_owned())
                })?;

            match mode {
                RepoFilesMoveMode::Copy => {
                    self.repo_files_service
                        .copy_file(&repo_id, &src_path, &dest_parent_path)
                        .await?
                }
                RepoFilesMoveMode::Move => {
                    self.repo_files_service
                        .move_file(&repo_id, &src_path, &dest_parent_path)
                        .await?
                }
            }
        }

        Ok(())
    }

    pub fn cancel(&self) {
        if let Some(dir_picker_id) = self.store.mutate(store::Event::RepoFilesMove, |state| {
            let dir_picker_id = state.repo_files_move.as_ref().map(|x| x.dir_picker_id);

            state.repo_files_move = None;

            dir_picker_id
        }) {
            self.repo_files_dir_pickers_service.destroy(dir_picker_id);
        }
    }

    pub fn check_create_dir(&self, name: &str) -> Result<(), CreateDirError> {
        self.store
            .with_state(|state| selectors::select_check_create_dir(state, name))
    }

    pub async fn create_dir(&self, name: &str) -> Result<(), CreateDirError> {
        let picker_id = self
            .store
            .with_state(|state| selectors::select_dir_picker_id(state))
            .ok_or_else(RepoFilesErrors::not_found)?;

        self.repo_files_dir_pickers_service
            .create_dir(picker_id, name)
            .await
    }
}
