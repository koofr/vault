use std::collections::HashMap;

use crate::common::state::Status;

use super::errors::RepoSpaceUsageError;

pub struct RepoSpaceUsageInfo<'a> {
    pub repo_id: &'a str,
    pub status: &'a Status<RepoSpaceUsageError>,
    pub space_used: Option<i64>,
}

#[derive(Clone)]
pub struct RepoSpaceUsage {
    pub repo_id: String,
    pub status: Status<RepoSpaceUsageError>,
    pub space_used: Option<i64>,
}

#[derive(Clone, Default)]
pub struct RepoSpaceUsagesState {
    pub usages: HashMap<u32, RepoSpaceUsage>,
    pub next_id: u32,
}
