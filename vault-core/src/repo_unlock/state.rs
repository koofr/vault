use std::collections::HashMap;

use crate::{
    common::state::Status,
    repos::{errors::UnlockRepoError, state::RepoUnlockMode},
    store::NextId,
    types::{DecryptedName, RepoId},
};

pub struct RepoUnlockInfo<'a> {
    pub repo_id: &'a RepoId,
    pub status: &'a Status<UnlockRepoError>,
    pub repo_name: Option<&'a DecryptedName>,
}

#[derive(Debug, Clone)]
pub struct RepoUnlockOptions {
    pub mode: RepoUnlockMode,
}

#[derive(Debug, Clone)]
pub struct RepoUnlock {
    pub repo_id: RepoId,
    pub mode: RepoUnlockMode,
    pub status: Status<UnlockRepoError>,
}

#[derive(Debug, Clone, Default)]
pub struct RepoUnlocksState {
    pub unlocks: HashMap<u32, RepoUnlock>,
    pub next_id: NextId,
}

impl RepoUnlocksState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
