use crate::{
    common::state::Status,
    repos::{
        errors::{RepoNotFoundError, UnlockRepoError},
        state::RepoConfig,
    },
    store,
    types::RepoId,
};

use super::state::RepoConfigBackup;

pub fn create(state: &mut store::State, notify: &store::Notify, repo_id: RepoId) -> u32 {
    notify(store::Event::RepoConfigBackup);

    let backup_id = state.repo_config_backups.next_id.next();

    let backup = RepoConfigBackup {
        repo_id,
        status: Status::Initial,
        config: None,
    };

    state.repo_config_backups.backups.insert(backup_id, backup);

    backup_id
}

pub fn generating(
    state: &mut store::State,
    notify: &store::Notify,
    backup_id: u32,
) -> Result<RepoId, UnlockRepoError> {
    let backup = match state.repo_config_backups.backups.get_mut(&backup_id) {
        Some(backup) => backup,
        None => return Err(UnlockRepoError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoConfigBackup);

    backup.status = Status::Loading {
        loaded: backup.status.loaded(),
    };

    Ok(backup.repo_id.clone())
}

pub fn generated(
    state: &mut store::State,
    notify: &store::Notify,
    backup_id: u32,
    res: Result<RepoConfig, UnlockRepoError>,
) -> Result<(), UnlockRepoError> {
    let backup = match state.repo_config_backups.backups.get_mut(&backup_id) {
        Some(backup) => backup,
        None => return Err(UnlockRepoError::RepoNotFound(RepoNotFoundError)),
    };

    notify(store::Event::RepoConfigBackup);

    match res {
        Ok(config) => {
            backup.status = Status::Loaded;
            backup.config = Some(config);
        }
        Err(err) => {
            backup.status = Status::Error {
                error: err,
                loaded: backup.status.loaded(),
            }
        }
    }

    Ok(())
}

pub fn destroy(state: &mut store::State, notify: &store::Notify, backup_id: u32) {
    notify(store::Event::RepoConfigBackup);

    state.repo_config_backups.backups.remove(&backup_id);
}
