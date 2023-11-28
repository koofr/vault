use crate::{
    common::state::Status,
    store,
    types::{DecryptedName, RepoId},
};

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
    repo_id: &RepoId,
) -> Result<&'a Repo, RepoNotFoundError> {
    state
        .repos
        .repos_by_id
        .get(repo_id)
        .ok_or(RepoNotFoundError)
}

pub fn select_repo_mut<'a>(
    state: &'a mut store::State,
    repo_id: &RepoId,
) -> Result<&'a mut Repo, RepoNotFoundError> {
    state
        .repos
        .repos_by_id
        .get_mut(repo_id)
        .ok_or(RepoNotFoundError)
}

pub fn select_repo_status<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
) -> (Result<&'a Repo, RepoNotFoundError>, Status<RepoInfoError>) {
    let repo = select_repo(state, repo_id);

    let status = match &state.repos.status {
        Status::Initial => Status::Initial,
        Status::Loading { loaded } => Status::Loading { loaded: *loaded },
        Status::Loaded => match repo {
            Ok(_) => Status::Loaded,
            Err(ref err) => Status::Error {
                error: RepoInfoError::RepoNotFound(err.clone()),
                loaded: true,
            },
        },
        Status::Error { error, loaded } => Status::Error {
            error: RepoInfoError::RemoteError(error.clone()),
            loaded: *loaded,
        },
    };

    (repo, status)
}

pub fn select_repo_info<'a>(state: &'a store::State, repo_id: &RepoId) -> RepoInfo<'a> {
    let (repo, status) = select_repo_status(state, repo_id);

    RepoInfo {
        status,
        repo: repo.ok(),
    }
}

pub fn select_repo_name<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
) -> Option<&'a DecryptedName> {
    select_repo(state, repo_id).ok().map(|repo| &repo.name)
}
