use std::collections::HashMap;

use crate::{common::state::Status, store::NextId};

use super::errors::RepoSpaceUsageError;

pub struct RepoSpaceUsageInfo<'a> {
    pub repo_id: &'a str,
    pub status: &'a Status<RepoSpaceUsageError>,
    pub space_used: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct RepoSpaceUsage {
    pub repo_id: String,
    pub status: Status<RepoSpaceUsageError>,
    pub space_used: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct RepoSpaceUsagesState {
    pub usages: HashMap<u32, RepoSpaceUsage>,
    pub next_id: NextId,
}

impl RepoSpaceUsagesState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
