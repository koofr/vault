use crate::{
    common::state::Status, rclone, remote::RemoteError, remote_files::state::RemoteFilesLocation,
    repos::state::RepoConfig,
};

use super::errors::RepoCreateError;

#[derive(Debug, Clone)]
pub struct RepoCreated {
    pub repo_id: String,
    pub config: RepoConfig,
}

#[derive(Debug, Clone)]
pub struct RepoCreateForm {
    pub init_status: Status<RemoteError>,
    pub primary_mount_id: Option<String>,
    pub location: Option<RemoteFilesLocation>,
    pub location_dir_picker_id: Option<u32>,
    pub password: String,
    pub salt: Option<String>,
    pub fill_from_rclone_config_error: Option<rclone::config::ParseConfigError>,
    pub create_status: Status<RepoCreateError>,
}

#[derive(Debug, Clone)]
pub enum RepoCreateState {
    Form(RepoCreateForm),
    Created(RepoCreated),
}
