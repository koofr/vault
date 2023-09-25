use std::collections::HashMap;

use crate::{
    common::state::Status,
    repo_unlock::state::RepoUnlockInfo,
    repos::{errors::UnlockRepoError, state::RepoConfig},
    store::NextId,
};

pub struct RepoConfigBackupInfo<'a> {
    pub unlock_info: RepoUnlockInfo<'a>,
    pub config: Option<&'a RepoConfig>,
}

#[derive(Debug, Clone)]
pub struct RepoConfigBackup {
    pub repo_id: String,
    pub status: Status<UnlockRepoError>,
    pub config: Option<RepoConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct RepoConfigBackupsState {
    pub backups: HashMap<u32, RepoConfigBackup>,
    pub next_id: NextId,
}

impl RepoConfigBackupsState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
