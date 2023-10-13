use std::collections::HashSet;

use crate::{
    cipher,
    files::{
        content_type::ext_to_content_type,
        file_category::{ext_to_file_category, FileCategory},
    },
    remote::RemoteError,
    remote_files::{selectors as remote_files_selectors, state::RemoteFile},
    repos::{errors::RepoNotFoundError, selectors as repos_selectors},
    store,
    utils::{name_utils, path_utils},
};

use super::{
    errors::{FileNameError, RenameFileError, RepoFilesErrors},
    state::{RepoFile, RepoFileType, RepoFilesBreadcrumb, RepoFilesSort, RepoFilesSortField},
};

pub fn get_file_id(repo_id: &str, path: &str) -> String {
    format!("{}:{}", repo_id, path)
}

pub fn get_error_file_id(repo_id: &str, path: &str) -> String {
    format!("err:{}:{}", repo_id, path)
}

pub fn get_file_unique_name(remote_file_unique_id: &str, ext: Option<&str>) -> String {
    match ext {
        Some(ext) => format!("{}.{}", remote_file_unique_id, ext),
        None => remote_file_unique_id.to_owned(),
    }
}

pub fn get_file_ext_content_type_category<'a>(
    name_lower: &'a str,
) -> (Option<String>, Option<String>, FileCategory) {
    let ext = name_utils::name_to_ext(name_lower);

    (
        ext.map(str::to_string),
        ext.and_then(ext_to_content_type).map(str::to_string),
        ext.and_then(ext_to_file_category)
            .unwrap_or(FileCategory::Generic),
    )
}

pub fn select_children<'a>(state: &'a store::State, file_id: &str) -> Option<&'a Vec<String>> {
    state.repo_files.children.get(file_id)
}

pub fn select_files<'a>(
    state: &'a store::State,
    repo_id: &str,
    path: &str,
) -> impl Iterator<Item = &'a RepoFile> {
    match select_children(state, &get_file_id(repo_id, path)) {
        Some(ids) => select_files_from_ids(state, ids),
        None => select_files_from_ids(state, &[]),
    }
}

pub fn select_files_from_ids<'a>(
    state: &'a store::State,
    ids: &'a [String],
) -> impl Iterator<Item = &'a RepoFile> {
    ids.iter().filter_map(|id| select_file(state, id))
}

pub fn select_file<'a>(state: &'a store::State, file_id: &str) -> Option<&'a RepoFile> {
    state.repo_files.files.get(file_id)
}

pub fn select_file_name<'a>(
    state: &'a store::State,
    file: &'a RepoFile,
) -> Result<&'a str, FileNameError> {
    match file.decrypted_path() {
        Ok("/") => Ok(
            repos_selectors::select_repo(state, &file.repo_id).map(|repo| repo.name.as_str())?
        ),
        Ok(_) => Ok(file.decrypted_name()?),
        Err(err) => Err(err.into()),
    }
}

pub fn select_remote_file<'a>(
    state: &'a store::State,
    file: &'a RepoFile,
) -> Option<&'a RemoteFile> {
    remote_files_selectors::select_file(
        state,
        &remote_files_selectors::get_file_id(&file.mount_id, &file.remote_path),
    )
}

pub fn select_repo_path_to_mount_path<'a>(
    state: &'a store::State,
    repo_id: &str,
    path: &str,
    cipher: &cipher::Cipher,
) -> Result<(String, String), RepoNotFoundError> {
    let repo = repos_selectors::select_repo(state, repo_id)?;

    let full_path = path_utils::join_paths(&repo.path, &cipher.encrypt_path(path));

    Ok((repo.mount_id.clone(), full_path))
}

pub fn select_mount_path_to_repo_id<'a>(
    state: &'a store::State,
    mount_id: &str,
    path: &str,
) -> Option<&'a str> {
    for path in path_utils::paths_chain(path) {
        if let Some(repo_id) = state
            .repos
            .repo_ids_by_remote_file_id
            .get(&remote_files_selectors::get_file_id(&mount_id, &path))
        {
            return Some(repo_id);
        }
    }

    None
}

pub fn select_is_root_loaded(state: &store::State, repo_id: &str, path: &str) -> bool {
    state
        .repo_files
        .loaded_roots
        .contains(&get_file_id(&repo_id, &path))
}

pub fn check_name_valid(name: &str) -> Result<(), RemoteError> {
    name_utils::validate_name(name).map_err(|_| RepoFilesErrors::invalid_path())
}

pub fn select_check_new_name_valid(
    state: &store::State,
    repo_id: &str,
    parent_path: &str,
    new_name: &str,
) -> Result<(), RemoteError> {
    check_name_valid(new_name)?;

    let new_path = path_utils::join_path_name(parent_path, new_name);

    match select_children(state, &get_file_id(repo_id, parent_path)) {
        Some(ids) => {
            if ids.contains(&get_file_id(repo_id, &new_path)) {
                Err(RepoFilesErrors::already_exists())
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
}

pub fn select_check_rename_file(
    state: &store::State,
    repo_id: &str,
    path: &str,
    name: &str,
) -> Result<(), RenameFileError> {
    let file =
        select_file(state, &get_file_id(repo_id, path)).ok_or_else(RepoFilesErrors::not_found)?;

    let path = file.decrypted_path()?;

    let parent_path = match path_utils::parent_path(path) {
        Some(parent_path) => parent_path,
        None => return Err(RenameFileError::RenameRoot),
    };

    select_check_new_name_valid(state, &file.repo_id, parent_path, name)?;

    Ok(())
}

pub fn select_breadcrumbs(
    state: &store::State,
    repo_id: &str,
    path: &str,
) -> Vec<RepoFilesBreadcrumb> {
    let repo = match repos_selectors::select_repo(state, repo_id) {
        Ok(repo) => repo,
        Err(_) => {
            return vec![];
        }
    };

    let paths = path_utils::paths_chain(path);
    let paths_len = paths.len();

    paths
        .into_iter()
        .enumerate()
        .map(|(i, path)| {
            let id = get_file_id(repo_id, &path);
            let name = match path_utils::path_to_name(&path) {
                Some(name) => name.to_owned(),
                None => repo.name.clone(),
            };

            RepoFilesBreadcrumb {
                id,
                repo_id: repo_id.to_owned(),
                path,
                name,
                last: i == paths_len - 1,
            }
        })
        .collect()
}

pub fn select_sorted_files(
    state: &store::State,
    file_ids: &[String],
    sort: &RepoFilesSort,
) -> Vec<String> {
    let RepoFilesSort { field, direction } = sort;

    let (mut dirs, mut files): (Vec<_>, Vec<_>) = file_ids
        .iter()
        .filter_map(|id| state.repo_files.files.get(id))
        .partition(|f| f.typ == RepoFileType::Dir);

    match field {
        RepoFilesSortField::Name => {
            dirs.sort_by(|a, b| direction.ordering(a.name_lower_force().cmp(b.name_lower_force())));
            files
                .sort_by(|a, b| direction.ordering(a.name_lower_force().cmp(b.name_lower_force())));
        }
        RepoFilesSortField::Size => {
            dirs.sort_by(|a, b| a.name_lower_force().cmp(b.name_lower_force()));
            files.sort_by(|a, b| direction.ordering(a.size_force().cmp(&b.size_force())));
        }
        RepoFilesSortField::Modified => {
            dirs.sort_by(|a, b| a.name_lower_force().cmp(b.name_lower_force()));
            files.sort_by(|a, b| direction.ordering(a.modified.cmp(&b.modified)));
        }
    }

    dirs.iter()
        .map(|file| file.id.clone())
        .chain(files.iter().map(|file| file.id.clone()))
        .collect()
}

pub fn select_used_names(
    state: &store::State,
    repo_id: &str,
    parent_path: &str,
) -> HashSet<String> {
    let mut used_names = HashSet::new();

    for f in select_files(state, repo_id, parent_path) {
        if let Ok(name) = f.decrypted_name() {
            used_names.insert(name.to_lowercase());
        }
    }

    used_names
}

pub fn get_unused_name(used_names: HashSet<String>, name: &str) -> String {
    name_utils::unused_name(name, |name| used_names.contains(&name.to_lowercase()))
}
