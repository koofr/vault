use crate::{
    common::state::Status,
    repos::errors::{RemoveRepoError, RepoNotFoundError},
    store,
};

use super::state::RepoRemove;

pub fn create(state: &mut store::State, notify: &store::Notify, repo_id: &str) -> u32 {
    notify(store::Event::RepoRemove);

    let remove_id = state.repo_removes.next_id.next();

    let remove = RepoRemove {
        repo_id: repo_id.to_owned(),
        status: Status::Initial,
    };

    state.repo_removes.removes.insert(remove_id, remove);

    remove_id
}

pub fn removing(
    state: &mut store::State,
    notify: &store::Notify,
    remove_id: u32,
) -> Result<String, RemoveRepoError> {
    let remove = match state.repo_removes.removes.get_mut(&remove_id) {
        Some(remove) => remove,
        None => return Err(RemoveRepoError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoRemove);

    remove.status = Status::Loading;

    Ok(remove.repo_id.clone())
}

pub fn removed(
    state: &mut store::State,
    notify: &store::Notify,
    remove_id: u32,
    res: Result<(), RemoveRepoError>,
) -> Result<(), RemoveRepoError> {
    let remove = match state.repo_removes.removes.get_mut(&remove_id) {
        Some(remove) => remove,
        None => return Err(RemoveRepoError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoRemove);

    remove.status = match res {
        Ok(()) => Status::Loaded,
        Err(err) => Status::Error { error: err },
    };

    Ok(())
}

pub fn destroy(state: &mut store::State, notify: &store::Notify, remove_id: u32) {
    notify(store::Event::RepoRemove);

    state.repo_removes.removes.remove(&remove_id);
}
