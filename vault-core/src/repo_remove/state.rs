use std::collections::HashMap;

use crate::{
    common::state::Status,
    repos::errors::RemoveRepoError,
    store::NextId,
    types::{DecryptedName, RepoId},
};

pub struct RepoRemoveInfo<'a> {
    pub repo_id: &'a RepoId,
    pub status: &'a Status<RemoveRepoError>,
    pub repo_name: Option<&'a DecryptedName>,
}

#[derive(Debug, Clone)]
pub struct RepoRemove {
    pub repo_id: RepoId,
    pub status: Status<RemoveRepoError>,
}

#[derive(Debug, Clone, Default)]
pub struct RepoRemovesState {
    pub removes: HashMap<u32, RepoRemove>,
    pub next_id: NextId,
}

impl RepoRemovesState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
