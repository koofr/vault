use crate::{
    common::state::Status,
    remote_files::state::RemoteFilesLocation,
    repos::{errors::RepoNotFoundError, selectors as repos_selectors},
    store,
};

use super::{errors::RepoSpaceUsageError, state::RepoSpaceUsage};

pub fn create(state: &mut store::State, notify: &store::Notify, repo_id: &str) -> u32 {
    notify(store::Event::RepoSpaceUsage);

    let usage_id = state.repo_space_usages.next_id;

    state.repo_space_usages.next_id += 1;

    let usage = RepoSpaceUsage {
        repo_id: repo_id.to_owned(),
        status: Status::Initial,
        space_used: None,
    };

    state.repo_space_usages.usages.insert(usage_id, usage);

    usage_id
}

pub fn calculating(
    state: &mut store::State,
    notify: &store::Notify,
    usage_id: u32,
) -> Result<RemoteFilesLocation, RepoSpaceUsageError> {
    let usage = match state.repo_space_usages.usages.get(&usage_id) {
        Some(usage) => usage,
        None => return Err(RepoSpaceUsageError::RepoNotFound(RepoNotFoundError)),
    };

    let location =
        repos_selectors::select_repo(state, &usage.repo_id).map(|repo| repo.get_location())?;

    let usage = match state.repo_space_usages.usages.get_mut(&usage_id) {
        Some(usage) => usage,
        None => return Err(RepoSpaceUsageError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoSpaceUsage);

    usage.status = Status::Loading;

    Ok(location)
}

pub fn calculated(
    state: &mut store::State,
    notify: &store::Notify,
    usage_id: u32,
    space_used: Option<i64>,
    res: Result<(), RepoSpaceUsageError>,
) -> Result<(), RepoSpaceUsageError> {
    let usage = match state.repo_space_usages.usages.get_mut(&usage_id) {
        Some(usage) => usage,
        None => return Err(RepoSpaceUsageError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoSpaceUsage);

    usage.status = match res {
        Ok(()) => Status::Loaded,
        Err(err) => Status::Error { error: err },
    };

    usage.space_used = space_used;

    Ok(())
}

pub fn destroy(state: &mut store::State, notify: &store::Notify, usage_id: u32) {
    notify(store::Event::RepoSpaceUsage);

    state.repo_space_usages.usages.remove(&usage_id);
}
