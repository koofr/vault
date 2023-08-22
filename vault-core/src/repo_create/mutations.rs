use crate::{
    common::state::Status, rclone, remote::RemoteError, remote_files::state::RemoteFilesLocation,
    store,
};

use super::{
    errors::RepoCreateError,
    state::{RepoCreateForm, RepoCreateState, RepoCreated},
};

pub const DEFAULT_REPO_NAME: &'static str = "My safe box";

pub fn init_loading(state: &mut store::State, salt: String) {
    state.repo_create = Some(RepoCreateState::Form(RepoCreateForm {
        init_status: Status::Loading,
        primary_mount_id: None,
        location: None,
        location_dir_picker_id: None,
        password: String::from(""),
        salt: Some(salt),
        fill_from_rclone_config_error: None,
        create_status: Status::Initial,
    }));
}

pub fn init_loaded(
    state: &mut store::State,
    status: Status<RemoteError>,
    primary_mount_id: Option<String>,
) {
    let no_existing_repos = state.repos.repos_by_id.is_empty();

    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return,
    };

    form.init_status = status;
    form.primary_mount_id = primary_mount_id;

    if no_existing_repos {
        if let Some(primary_mount_id) = &form.primary_mount_id {
            form.location = Some(RemoteFilesLocation {
                mount_id: primary_mount_id.to_owned(),
                path: format!("/{}", DEFAULT_REPO_NAME),
            });
        }
    }
}

pub fn reset(state: &mut store::State) {
    state.repo_create = None;
}

pub fn set_location(state: &mut store::State, location: RemoteFilesLocation) {
    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return,
    };

    form.location = Some(location);
}

pub fn location_dir_picker_show(state: &mut store::State, location_dir_picker_id: u32) {
    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return,
    };

    form.location_dir_picker_id = Some(location_dir_picker_id);
}

pub fn location_dir_picker_cancel(state: &mut store::State) -> Option<u32> {
    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return None,
    };

    let location_dir_picker_id = form.location_dir_picker_id;

    form.location_dir_picker_id = None;

    location_dir_picker_id
}

pub fn set_password(state: &mut store::State, password: String) {
    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return,
    };

    form.password = password;
}

pub fn set_salt(state: &mut store::State, salt: Option<String>) {
    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return,
    };

    form.salt = salt;
}

pub fn fill_from_rclone_config(
    state: &mut store::State,
    config: Result<rclone::config::Config, rclone::config::ParseConfigError>,
) {
    let form = match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => form,
        _ => return,
    };

    match config {
        Ok(config) => {
            let rclone::config::Config {
                path,
                password,
                salt,
                ..
            } = config;

            if let Some(primary_mount_id) = &form.primary_mount_id {
                form.location = Some(RemoteFilesLocation {
                    mount_id: primary_mount_id.to_owned(),
                    path,
                });
            }

            form.password = password;
            form.salt = salt;

            form.fill_from_rclone_config_error = None;
        }
        Err(err) => {
            form.fill_from_rclone_config_error = Some(err);
        }
    }
}

pub fn repo_creating(state: &mut store::State) {
    match state.repo_create {
        Some(RepoCreateState::Form(ref mut form)) => {
            form.create_status = Status::Loading;
        }
        _ => (),
    };
}

pub fn repo_create(state: &mut store::State, res: Result<RepoCreated, RepoCreateError>) {
    match res {
        Ok(created) => state.repo_create = Some(RepoCreateState::Created(created)),
        Err(err) => {
            match state.repo_create {
                Some(RepoCreateState::Form(ref mut form)) => {
                    form.create_status = Status::Error { error: err };
                }
                _ => (),
            };
        }
    }
}
