use crate::{common::state::Status, store};

use super::{
    errors::{RepoInfoError, RepoNotFoundError},
    state::{Repo, RepoInfo},
};

pub fn select_repos<'a>(state: &'a store::State) -> Vec<&'a Repo> {
    let mut repos: Vec<&'a Repo> = state
        .repos
        .repos_by_id
        .iter()
        .map(|(_, repo)| repo)
        .collect();

    repos.sort_by(|a, b| a.added.partial_cmp(&b.added).unwrap());

    repos
}

pub fn select_repo<'a>(
    state: &'a store::State,
    repo_id: &str,
) -> Result<&'a Repo, RepoNotFoundError> {
    state
        .repos
        .repos_by_id
        .get(repo_id)
        .ok_or(RepoNotFoundError)
}

pub fn select_repo_info<'a>(state: &'a store::State, repo_id: &str) -> RepoInfo<'a> {
    let repo = select_repo(state, repo_id);

    let status = match &state.repos.status {
        Status::Initial => Status::Initial,
        Status::Loading => Status::Loading,
        Status::Loaded => match repo {
            Ok(_) => Status::Loaded,
            Err(ref err) => Status::Error {
                error: RepoInfoError::RepoNotFound(err.clone()),
            },
        },
        Status::Reloading => Status::Reloading,
        Status::Error { error } => Status::Error {
            error: RepoInfoError::RemoteError(error.clone()),
        },
    };

    RepoInfo {
        status,
        repo: repo.ok(),
    }
}

pub fn select_repo_name<'a>(state: &'a store::State, repo_id: &str) -> Option<&'a str> {
    select_repo(state, repo_id)
        .ok()
        .map(|repo| repo.name.as_str())
}
