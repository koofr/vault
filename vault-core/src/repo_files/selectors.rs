use crate::{
    cipher, remote::RemoteError, remote_files::selectors as remote_files_selectors,
    repos::errors::RepoNotFoundError, repos::selectors as repos_selectors, repos::state::Repo,
    store, utils::path_utils,
};

use super::{
    errors::{RenameFileError, RepoFilesErrors, RepoMountPathToPathError},
    state::{RepoFile, RepoFileName, RepoFileType, RepoFilesBreadcrumb},
};

pub fn get_file_id(repo_id: &str, path: &str) -> String {
    format!("{}:{}", repo_id, path)
}

pub fn repo_file_sort_key<'a>(file: &'a RepoFile) -> (&'a RepoFileType, &'a str) {
    (
        &file.typ,
        match &file.name {
            RepoFileName::Decrypted {
                name: _,
                name_lower,
            } => name_lower,
            RepoFileName::DecryptError {
                encrypted_name: _,
                encrypted_name_lower,
                error: _,
            } => encrypted_name_lower,
        },
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

pub fn select_file_name<'a>(state: &'a store::State, file: &'a RepoFile) -> Option<&'a str> {
    match file.decrypted_path() {
        Ok("/") => repos_selectors::select_repo(state, &file.repo_id)
            .ok()
            .map(|repo| repo.name.as_str()),
        Ok(_) => match file.decrypted_name() {
            Ok(name) => Some(name),
            _ => None,
        },
        _ => None,
    }
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

pub fn select_repo_mount_path_to_path<'a>(
    state: &'a store::State,
    repo_id: &str,
    mount_path: &str,
    cipher: &cipher::Cipher,
) -> Result<(&'a Repo, String), RepoMountPathToPathError> {
    let repo = repos_selectors::select_repo(state, repo_id)
        .map_err(RepoMountPathToPathError::RepoNotFound)?;

    let path = if repo.path == mount_path {
        String::from("/")
    } else {
        cipher
            .decrypt_path(if repo.path == "/" {
                &mount_path
            } else {
                &mount_path[repo.path.len()..]
            })
            .map_err(RepoMountPathToPathError::DecryptFilenameError)?
    };

    Ok((repo, path))
}

pub fn select_is_root_loaded(state: &store::State, repo_id: &str, path: &str) -> bool {
    state
        .repo_files
        .loaded_roots
        .contains(&get_file_id(&repo_id, &path))
}

pub fn check_name_valid(name: &str) -> Result<(), RemoteError> {
    if !name.is_empty() && !name.contains('/') {
        Ok(())
    } else {
        Err(RepoFilesErrors::invalid_path())
    }
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

    let file_name = file.decrypted_name()?;

    // case change
    if name != file_name && name.to_lowercase() == file_name.to_lowercase() {
        return Ok(());
    }

    let path = file.decrypted_path()?;

    select_check_new_name_valid(state, &file.repo_id, path, name)?;

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

#[cfg(test)]
mod tests {
    use crate::{
        cipher::test_helpers as cipher_test_helpers,
        remote::test_helpers as remote_test_helpers,
        repo_files::selectors::{select_mount_path_to_repo_id, select_repo_mount_path_to_path},
        repos::mutations as repos_mutations,
        store,
    };

    #[test]
    fn test_select_repo_mount_path_to_path_root() {
        let mut state = store::State::default();
        let repo = remote_test_helpers::create_repo("r1", "m1", "/");
        repos_mutations::repos_loaded(&mut state, vec![repo.clone()]);
        let cipher = cipher_test_helpers::create_cipher();
        let select = |mount_path: &str| {
            let repo_id = select_mount_path_to_repo_id(&state, &repo.mount_id, mount_path).unwrap();

            select_repo_mount_path_to_path(&state, &repo_id, mount_path, &cipher)
                .map(|(_, path)| path)
        };

        assert_eq!(select("/"), Ok(format!("/")));
        assert_eq!(
            select(&cipher.encrypt_path("/foo")),
            Ok(String::from("/foo"))
        );
        assert_eq!(
            select(&cipher.encrypt_path("/foo/bar")),
            Ok(String::from("/foo/bar"))
        );
    }

    #[test]
    fn test_select_repo_mount_path_to_path_child() {
        let mut state = store::State::default();
        let repo = remote_test_helpers::create_repo("r1", "m1", "/Vault");
        repos_mutations::repos_loaded(&mut state, vec![repo.clone()]);
        let cipher = cipher_test_helpers::create_cipher();
        let select = |mount_path: &str| {
            let repo_id = select_mount_path_to_repo_id(&state, &repo.mount_id, mount_path).unwrap();

            select_repo_mount_path_to_path(&state, &repo_id, mount_path, &cipher)
                .map(|(_, path)| path)
        };

        assert_eq!(select("/Vault"), Ok(format!("/")));
        assert_eq!(
            select(&format!("/Vault{}", cipher.encrypt_path("/foo"))),
            Ok(String::from("/foo"))
        );
        assert_eq!(
            select(&format!("/Vault{}", cipher.encrypt_path("/foo/bar"))),
            Ok(String::from("/foo/bar"))
        );
    }
}
