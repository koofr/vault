use crate::{
    repo_files::errors::MoveFileError,
    store,
    types::{EncryptedPath, RepoId},
};

use super::{
    selectors,
    state::{RepoFilesMoveInfo, RepoFilesMoveMode, RepoFilesMoveState},
};

pub fn show(
    state: &mut store::State,
    notify: &store::Notify,
    repo_id: RepoId,
    src_paths: Vec<EncryptedPath>,
    dest_path: EncryptedPath,
    mode: RepoFilesMoveMode,
    dir_picker_id: u32,
) {
    notify(store::Event::RepoFilesMove);

    state.repo_files_move = Some(RepoFilesMoveState {
        repo_id,
        src_paths,
        mode,
        dest_path,
        dir_picker_id,
    });
}

pub fn set_dest_path(state: &mut store::State, notify: &store::Notify, dest_path: EncryptedPath) {
    if let Some(ref mut repo_files_move) = state.repo_files_move {
        notify(store::Event::RepoFilesMove);

        repo_files_move.dest_path = dest_path;
    }
}

pub fn move_files(
    state: &mut store::State,
    notify: &store::Notify,
) -> Result<RepoFilesMoveInfo, MoveFileError> {
    notify(store::Event::RepoFilesMove);

    let (files_move_state, dest_path) = selectors::select_check_move(state)?;

    let info = RepoFilesMoveInfo {
        repo_id: files_move_state.repo_id.clone(),
        src_paths: files_move_state.src_paths.clone(),
        mode: files_move_state.mode.clone(),
        dir_picker_id: files_move_state.dir_picker_id,
        dest_path: dest_path.to_owned(),
    };

    state.repo_files_move = None;

    Ok(info)
}

pub fn cancel(state: &mut store::State, notify: &store::Notify) -> Option<u32> {
    notify(store::Event::RepoFilesMove);

    let dir_picker_id = state.repo_files_move.as_ref().map(|x| x.dir_picker_id);

    state.repo_files_move = None;

    dir_picker_id
}
