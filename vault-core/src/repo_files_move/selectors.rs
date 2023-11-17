use crate::{
    repo_files::{
        errors::{MoveFileError, RepoFilesErrors},
        selectors as repo_files_selectors,
        state::RepoFile,
    },
    store,
    types::{DecryptedPath, RepoFileId, RepoId},
    utils::repo_path_utils,
};

use super::{errors::DirPickerClickError, state::RepoFilesMoveState};

pub fn select_dir_picker_id(state: &store::State) -> Option<u32> {
    state.repo_files_move.as_ref().map(|x| x.dir_picker_id)
}

pub fn select_dest_path<'a>(state: &'a store::State) -> Option<&'a DecryptedPath> {
    state.repo_files_move.as_ref().map(|x| &x.dest_path)
}

pub fn select_repo_id_dest_path<'a>(
    state: &'a store::State,
) -> Option<(&'a RepoId, &'a DecryptedPath)> {
    state
        .repo_files_move
        .as_ref()
        .map(|x| (&x.repo_id, &x.dest_path))
}

pub fn select_dest_file<'a>(state: &'a store::State) -> Option<&'a RepoFile> {
    select_repo_id_dest_path(state).and_then(|(repo_id, dest_path)| {
        repo_files_selectors::select_file(
            state,
            &repo_files_selectors::get_file_id(repo_id, dest_path),
        )
    })
}

pub fn select_dir_picker_click(
    state: &store::State,
    file_id: &RepoFileId,
) -> Result<(u32, DecryptedPath), DirPickerClickError> {
    let dir_picker_id = select_dir_picker_id(state).ok_or_else(RepoFilesErrors::not_found)?;

    let file =
        repo_files_selectors::select_file(state, file_id).ok_or_else(RepoFilesErrors::not_found)?;

    let path = file.decrypted_path()?.to_owned();

    Ok((dir_picker_id, path))
}

pub fn select_create_dir_enabled(state: &store::State) -> bool {
    select_dest_file(state)
        .map(|file| file.typ.is_dir())
        .unwrap_or(false)
}

pub fn select_check_move<'a>(
    state: &'a store::State,
) -> Result<(&'a RepoFilesMoveState, &'a DecryptedPath), MoveFileError> {
    let files_move_state = state
        .repo_files_move
        .as_ref()
        .ok_or_else(RepoFilesErrors::not_found)?;

    let dest_parent_path = select_dest_path(state).ok_or_else(RepoFilesErrors::not_found)?;

    for src_path in files_move_state.src_paths.iter() {
        let (_, src_name) = repo_path_utils::split_parent_name(src_path)
            .ok_or_else(RepoFilesErrors::invalid_path)?;
        let dest_path = repo_path_utils::join_path_name(dest_parent_path, &src_name);
        let dest_file_id = repo_files_selectors::get_file_id(&files_move_state.repo_id, &dest_path);

        if src_path == &dest_path {
            return Err(RepoFilesErrors::move_into_self().into());
        }

        if repo_files_selectors::select_file(state, &dest_file_id).is_some() {
            return Err(RepoFilesErrors::already_exists().into());
        }
    }

    Ok((files_move_state, dest_parent_path))
}
