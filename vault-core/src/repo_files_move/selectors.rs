use crate::{
    repo_files::{
        errors::{CreateDirError, MoveFileError, RepoFilesErrors},
        state::RepoFile,
    },
    repo_files_dir_pickers::selectors as repo_files_dir_pickers_selectors,
    store,
};

pub fn select_dir_picker_id(state: &store::State) -> Option<u32> {
    state.repo_files_move.as_ref().map(|x| x.dir_picker_id)
}

pub fn select_dest_file<'a>(state: &'a store::State) -> Option<&'a RepoFile> {
    select_dir_picker_id(state).and_then(|picker_id| {
        repo_files_dir_pickers_selectors::select_selected_file(state, picker_id)
    })
}

pub fn select_can_show_create_dir(state: &store::State) -> bool {
    select_dir_picker_id(state)
        .map(|picker_id| {
            repo_files_dir_pickers_selectors::select_can_show_create_dir(state, picker_id)
        })
        .unwrap_or(false)
}

pub fn select_check_create_dir(state: &store::State, name: &str) -> Result<(), CreateDirError> {
    let picker_id = select_dir_picker_id(state).ok_or_else(RepoFilesErrors::not_found)?;

    repo_files_dir_pickers_selectors::select_check_create_dir(state, picker_id, name)?;

    Ok(())
}

pub fn select_check_move<'a>(state: &'a store::State) -> Result<(), MoveFileError> {
    // TODO check that the selected file is not a parent of any source files
    select_dest_file(state).ok_or_else(RepoFilesErrors::not_found)?;

    Ok(())
}
