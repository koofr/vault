use std::collections::HashMap;

use crate::{common::state::Status, remote::RemoteError, remote_files::state::RemoteFilesLocation};

use super::{errors::RepoInfoError, repo_tree::RepoTree};

#[derive(Debug, Clone, Copy)]
pub enum RepoState {
    Locked,
    Unlocked,
}

impl RepoState {
    pub fn is_locked(&self) -> bool {
        match self {
            Self::Locked => true,
            Self::Unlocked => false,
        }
    }

    pub fn is_unlocked(&self) -> bool {
        !self.is_locked()
    }
}

#[derive(Debug, Clone)]
pub struct Repo {
    pub id: String,
    pub name: String,
    pub mount_id: String,
    pub path: String,
    pub salt: Option<String>,
    pub added: i64,
    pub password_validator: String,
    pub password_validator_encrypted: String,
    pub web_url: String,
    pub state: RepoState,
}

impl Repo {
    pub fn get_location(&self) -> RemoteFilesLocation {
        RemoteFilesLocation {
            mount_id: self.mount_id.clone(),
            path: self.path.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepoInfo<'a> {
    pub status: Status<RepoInfoError>,
    pub repo: Option<&'a Repo>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoConfig {
    pub name: String,
    pub location: RemoteFilesLocation,
    pub password: String,
    pub salt: Option<String>,
    pub rclone_config: String,
}

#[derive(Debug, Clone)]
pub enum RepoUnlockMode {
    Unlock,
    Verify,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoCreated {
    pub repo_id: String,
    pub config: RepoConfig,
}

#[derive(Debug, Clone, Default)]
pub struct ReposState {
    pub status: Status<RemoteError>,
    pub repos_by_id: HashMap<String, Repo>,
    pub repo_ids_by_remote_file_id: HashMap<String, String>,
    pub mount_repo_trees: HashMap<String, RepoTree>,
}

impl ReposState {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReposMutationState {
    pub locked_repos: Vec<String>,
    pub unlocked_repos: Vec<String>,
    pub removed_repos: Vec<String>,
}
