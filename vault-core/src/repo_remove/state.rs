use crate::{common::state::Status, repos::errors::RemoveRepoError};

pub struct RepoRemoveInfo<'a> {
    pub repo_id: &'a str,
    pub status: &'a Status<RemoveRepoError>,
    pub repo_name: Option<&'a str>,
}

#[derive(Clone)]
pub struct RepoRemoveState {
    pub repo_id: String,
    pub status: Status<RemoveRepoError>,
}
