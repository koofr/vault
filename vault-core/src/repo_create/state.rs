use std::collections::HashMap;

use crate::{
    common::state::Status,
    rclone,
    remote::RemoteError,
    remote_files::state::RemoteFilesLocation,
    repos::{errors::CreateRepoError, state::RepoCreated},
};

#[derive(Debug, Clone)]
pub struct RepoCreateForm {
    pub create_load_status: Status<RemoteError>,
    pub primary_mount_id: Option<String>,
    pub location: Option<RemoteFilesLocation>,
    pub location_dir_picker_id: Option<u32>,
    pub password: String,
    pub salt: Option<String>,
    pub fill_from_rclone_config_error: Option<rclone::config::ParseConfigError>,
    pub create_repo_status: Status<CreateRepoError>,
}

#[derive(Debug, Clone)]
pub enum RepoCreate {
    Form(RepoCreateForm),
    Created(RepoCreated),
}

#[derive(Debug, Clone, Default)]
pub struct RepoCreatesState {
    pub creates: HashMap<u32, RepoCreate>,
    pub next_id: u32,
}
