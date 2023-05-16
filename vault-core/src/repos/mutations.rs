use urlencoding::encode;

use crate::{
    remote::models, remote_files::selectors as remote_files_selectors,
    repo_files::selectors as repo_files_selectors, store,
};

use super::{
    errors::RepoNotFoundError,
    selectors::select_repo,
    state::{Repo, RepoState},
};

fn vault_repo_to_repo(repo: models::VaultRepo, base_url: &str) -> Repo {
    let models::VaultRepo {
        id,
        name,
        mount_id,
        path,
        salt,
        password_validator,
        password_validator_encrypted,
        added,
    } = repo;

    let web_url = format!(
        "{}/app/storage/{}?path={}",
        base_url,
        &mount_id,
        encode(&path)
    );

    Repo {
        id,
        name,
        mount_id,
        path,
        salt,
        added,
        password_validator,
        password_validator_encrypted,
        state: RepoState::Locked,
        web_url,
    }
}

pub fn repo_loaded(state: &mut store::State, repo: models::VaultRepo) {
    let repo = vault_repo_to_repo(repo, &state.config.base_url);

    state.repos.repo_ids_by_remote_file_id.insert(
        remote_files_selectors::get_file_id(&repo.mount_id, &repo.path),
        repo.id.clone(),
    );

    state.repos.repos_by_id.insert(repo.id.clone(), repo);
}

pub fn repos_loaded(state: &mut store::State, repos: Vec<models::VaultRepo>) {
    state.repos.repos_by_id.clear();

    for repo in repos {
        repo_loaded(state, repo);
    }
}

pub fn lock_repo(state: &mut store::State, repo_id: &str) -> Result<(), RepoNotFoundError> {
    let file_id_prefix = repo_files_selectors::get_file_id(repo_id, "");

    state
        .repo_files
        .files
        .retain(|key, _| !key.starts_with(&file_id_prefix));

    state
        .repo_files
        .children
        .retain(|key, _| !key.starts_with(&file_id_prefix));

    match state.repos.repos_by_id.get_mut(repo_id) {
        Some(repo) => {
            repo.state = RepoState::Locked;

            Ok(())
        }
        None => Err(RepoNotFoundError),
    }
}

pub fn unlock_repo(state: &mut store::State, repo_id: &str) -> Result<(), RepoNotFoundError> {
    match state.repos.repos_by_id.get_mut(repo_id) {
        Some(repo) => {
            repo.state = RepoState::Unlocked;

            Ok(())
        }
        None => Err(RepoNotFoundError),
    }
}

pub fn remove_repo(state: &mut store::State, repo_id: &str) {
    if let Some((mount_id, path)) = select_repo(state, repo_id)
        .map(|repo| (repo.mount_id.clone(), repo.path.clone()))
        .ok()
    {
        state
            .repos
            .repo_ids_by_remote_file_id
            .remove(&remote_files_selectors::get_file_id(&mount_id, &path));
    }

    state.repos.repos_by_id.remove(repo_id);
}
