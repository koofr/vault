use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    cipher::Cipher,
    common::state::Status,
    remote::RemoteError,
    remote_files::state::RemoteFilesLocation,
    types::{DecryptedName, MountId, RemoteFileId, RemotePath, RepoId, TimeMillis},
};

use super::{errors::RepoInfoError, repo_tree::RepoTree};

#[derive(Debug, Clone)]
pub enum RepoState {
    Locked,
    Unlocked { cipher: Arc<Cipher> },
}

impl RepoState {
    pub fn is_locked(&self) -> bool {
        match self {
            Self::Locked => true,
            Self::Unlocked { .. } => false,
        }
    }

    pub fn is_unlocked(&self) -> bool {
        !self.is_locked()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAutoLock {
    pub after: Option<RepoAutoLockAfter>,
    pub on_app_hidden: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum RepoAutoLockAfter {
    Inactive1Minute,
    Inactive5Mininutes,
    Inactive10Minutes,
    Inactive30Minutes,
    Inactive1Hour,
    Inactive2Hours,
    Inactive4Hours,
    Custom(Duration),
}

impl Into<Duration> for RepoAutoLockAfter {
    fn into(self) -> Duration {
        match self {
            Self::Inactive1Minute => Duration::from_secs(1 * 60),
            Self::Inactive5Mininutes => Duration::from_secs(5 * 60),
            Self::Inactive10Minutes => Duration::from_secs(10 * 60),
            Self::Inactive30Minutes => Duration::from_secs(30 * 60),
            Self::Inactive1Hour => Duration::from_secs(60 * 60),
            Self::Inactive2Hours => Duration::from_secs(120 * 60),
            Self::Inactive4Hours => Duration::from_secs(240 * 60),
            Self::Custom(duration) => duration.clone(),
        }
    }
}

impl From<Duration> for RepoAutoLockAfter {
    fn from(duration: Duration) -> Self {
        match duration.as_secs() {
            60 => Self::Inactive1Minute,
            300 => Self::Inactive5Mininutes,
            600 => Self::Inactive10Minutes,
            1800 => Self::Inactive30Minutes,
            3600 => Self::Inactive1Hour,
            7200 => Self::Inactive2Hours,
            14400 => Self::Inactive4Hours,
            _ => Self::Custom(duration),
        }
    }
}

impl Serialize for RepoAutoLockAfter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let duration: Duration = (*self).into();

        // f64 for json compatibility
        serializer.serialize_f64(duration.as_secs_f64())
    }
}

impl<'de> Deserialize<'de> for RepoAutoLockAfter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // f64 for json compatibility
        let value: f64 = Deserialize::deserialize(deserializer)?;

        let duration = Duration::from_secs_f64(value);

        Ok(duration.into())
    }
}

#[derive(Debug, Clone)]
pub struct Repo {
    pub id: RepoId,
    pub name: DecryptedName,
    pub mount_id: MountId,
    pub path: RemotePath,
    pub salt: Option<String>,
    pub added: i64,
    pub password_validator: String,
    pub password_validator_encrypted: String,
    pub web_url: String,
    pub state: RepoState,
    pub last_activity: Option<TimeMillis>,
    pub auto_lock: Option<RepoAutoLock>,
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
    pub default_auto_lock: &'a RepoAutoLock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepoConfig {
    pub name: DecryptedName,
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
    pub repo_id: RepoId,
    pub config: RepoConfig,
}

#[derive(Debug, Clone, Default)]
pub struct ReposState {
    pub status: Status<RemoteError>,
    pub repos_by_id: HashMap<RepoId, Repo>,
    pub repo_ids_by_remote_file_id: HashMap<RemoteFileId, RepoId>,
    pub mount_repo_trees: HashMap<MountId, RepoTree>,
}

impl ReposState {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReposMutationState {
    pub locked_repos: Vec<(RepoId, Arc<Cipher>)>,
    pub unlocked_repos: Vec<(RepoId, Arc<Cipher>)>,
    pub removed_repos: Vec<RepoId>,
}
