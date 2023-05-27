use std::collections::HashMap;

use crate::{common::state::Status, repos::errors::UnlockRepoError};

pub struct RepoUnlockInfo<'a> {
    pub repo_id: &'a str,
    pub status: &'a Status<UnlockRepoError>,
    pub repo_name: Option<&'a str>,
}

#[derive(Clone)]
pub struct RepoUnlock {
    pub repo_id: String,
    pub status: Status<UnlockRepoError>,
}

#[derive(Clone, Default)]
pub struct RepoUnlocksState {
    pub unlocks: HashMap<u32, RepoUnlock>,
    pub next_id: u32,
}
