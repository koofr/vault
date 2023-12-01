use std::collections::HashMap;

use crate::{
    common::state::Status,
    rclone,
    remote_files::state::RemoteFilesLocation,
    repos::{errors::CreateRepoError, state::RepoCreated},
    store::NextId,
    types::MountId,
};

use super::errors::CreateLoadError;

#[derive(Debug, Clone, PartialEq)]
pub struct RepoCreateForm {
    pub create_load_status: Status<CreateLoadError>,
    pub primary_mount_id: Option<MountId>,
    pub location: Option<RemoteFilesLocation>,
    pub location_dir_picker_id: Option<u32>,
    pub password: String,
    pub salt: Option<String>,
    pub fill_from_rclone_config_error: Option<rclone::config::ParseConfigError>,
    pub create_repo_status: Status<CreateRepoError>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepoCreate {
    Form(RepoCreateForm),
    Created(RepoCreated),
}

impl RepoCreate {
    pub fn form(&self) -> Option<&RepoCreateForm> {
        match self {
            Self::Form(form) => Some(form),
            Self::Created(_) => None,
        }
    }

    pub fn created(&self) -> Option<&RepoCreated> {
        match self {
            Self::Created(created) => Some(created),
            Self::Form(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RepoCreatesState {
    pub creates: HashMap<u32, RepoCreate>,
    pub next_id: NextId,
}

impl RepoCreatesState {
    pub fn reset(&mut self) {
        *self = Self {
            next_id: self.next_id.clone(),
            ..Default::default()
        };
    }
}
