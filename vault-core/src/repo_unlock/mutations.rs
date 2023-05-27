use crate::{
    common::state::Status,
    repos::errors::{RepoNotFoundError, UnlockRepoError},
    store,
};

use super::state::RepoUnlock;

pub fn create(state: &mut store::State, notify: &store::Notify, repo_id: &str) -> u32 {
    notify(store::Event::RepoUnlock);

    let unlock_id = state.repo_unlocks.next_id;

    state.repo_unlocks.next_id += 1;

    let repo_unlock = RepoUnlock {
        repo_id: repo_id.to_owned(),
        status: Status::Initial,
    };

    state.repo_unlocks.unlocks.insert(unlock_id, repo_unlock);

    unlock_id
}

pub fn unlocking(
    state: &mut store::State,
    notify: &store::Notify,
    unlock_id: u32,
) -> Result<String, UnlockRepoError> {
    let repo_unlock = match state.repo_unlocks.unlocks.get_mut(&unlock_id) {
        Some(repo_unlock) => repo_unlock,
        None => return Err(UnlockRepoError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoUnlock);

    repo_unlock.status = Status::Loading;

    Ok(repo_unlock.repo_id.clone())
}

pub fn unlocked(
    state: &mut store::State,
    notify: &store::Notify,
    unlock_id: u32,
    res: Result<(), UnlockRepoError>,
) {
    let repo_unlock = match state.repo_unlocks.unlocks.get_mut(&unlock_id) {
        Some(repo_unlock) => repo_unlock,
        None => return,
    };

    notify(store::Event::RepoUnlock);

    repo_unlock.status = match &res {
        Ok(()) => Status::Loaded,
        Err(err) => Status::Error { error: err.clone() },
    };
}

pub fn destroy(state: &mut store::State, notify: &store::Notify, unlock_id: u32) {
    notify(store::Event::RepoUnlock);

    state.repo_unlocks.unlocks.remove(&unlock_id);
}
