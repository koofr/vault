use crate::{
    common::state::Status,
    repos::{errors::RepoConfigError, state::RepoConfig},
};

pub struct RepoConfigBackupInfo<'a> {
    pub repo_id: &'a str,
    pub status: &'a Status<RepoConfigError>,
    pub config: Option<&'a RepoConfig>,
}

#[derive(Clone)]
pub struct RepoConfigBackupState {
    pub repo_id: String,
    pub status: Status<RepoConfigError>,
    pub config: Option<RepoConfig>,
}
