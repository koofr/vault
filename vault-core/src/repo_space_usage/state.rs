use crate::common::state::Status;

use super::errors::RepoSpaceUsageError;

pub struct RepoSpaceUsageInfo<'a> {
    pub repo_id: &'a str,
    pub status: &'a Status<RepoSpaceUsageError>,
    pub space_used: Option<i64>,
}

#[derive(Clone)]
pub struct RepoSpaceUsageState {
    pub repo_id: String,
    pub status: Status<RepoSpaceUsageError>,
    pub space_used: Option<i64>,
}
