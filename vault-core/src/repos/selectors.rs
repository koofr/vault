use crate::store;

use super::{errors::RepoNotFoundError, state::Repo};

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
