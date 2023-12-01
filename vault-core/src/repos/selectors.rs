use std::{collections::HashMap, sync::Arc};

use crate::{
    cipher::Cipher,
    common::state::Status,
    store,
    types::{DecryptedName, RepoId},
};

use super::{
    errors::{GetCipherError, RepoInfoError, RepoLockedError, RepoNotFoundError},
    state::{Repo, RepoAutoLock, RepoInfo, RepoState},
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
    let default_auto_lock = select_default_auto_lock(state);

    RepoInfo {
        status,
        repo: repo.ok(),
        default_auto_lock,
    }
}

pub fn select_repo_name<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
) -> Option<&'a DecryptedName> {
    select_repo(state, repo_id).ok().map(|repo| &repo.name)
}

pub fn select_cipher<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
) -> Result<&'a Cipher, GetCipherError> {
    select_repo(state, repo_id)
        .map_err(Into::into)
        .and_then(|repo| match &repo.state {
            RepoState::Locked => Err(GetCipherError::RepoLocked(RepoLockedError)),
            RepoState::Unlocked { cipher } => Ok(cipher.as_ref()),
        })
}

pub fn select_cipher_owned(
    state: &store::State,
    repo_id: &RepoId,
) -> Result<Arc<Cipher>, GetCipherError> {
    select_repo(state, repo_id)
        .map_err(Into::into)
        .and_then(|repo| match &repo.state {
            RepoState::Locked => Err(GetCipherError::RepoLocked(RepoLockedError)),
            RepoState::Unlocked { cipher } => Ok(cipher.clone()),
        })
}

pub fn select_auto_locks(state: &store::State) -> HashMap<RepoId, RepoAutoLock> {
    state
        .repos
        .repos_by_id
        .iter()
        .filter_map(|(repo_id, repo)| {
            repo.auto_lock
                .clone()
                .map(|auto_lock| (repo_id.to_owned(), auto_lock))
        })
        .collect()
}

pub fn select_default_auto_lock<'a>(state: &'a store::State) -> &'a RepoAutoLock {
    &state.config.repos.default_auto_lock
}
