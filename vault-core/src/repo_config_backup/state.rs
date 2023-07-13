use std::collections::HashMap;

use crate::{
    common::state::Status,
    repo_unlock::state::RepoUnlockInfo,
    repos::{errors::UnlockRepoError, state::RepoConfig},
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
    pub next_id: u32,
}
