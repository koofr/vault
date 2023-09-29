use crate::{
    common::state::Status,
    rclone, remote,
    remote_files::state::RemoteFilesLocation,
    repos::{errors::CreateRepoError, state::RepoCreated},
    store,
};

use super::{
    selectors,
    state::{RepoCreate, RepoCreateForm},
};

pub const DEFAULT_REPO_NAME: &'static str = "My safe box";

pub fn create(state: &mut store::State, notify: &store::Notify, salt: String) -> u32 {
    notify(store::Event::RepoCreate);

    let create_id = state.repo_creates.next_id.next();

    let repo_create = RepoCreate::Form(RepoCreateForm {
        create_load_status: Status::Loading { loaded: false },
        primary_mount_id: None,
        location: None,
        location_dir_picker_id: None,
        password: String::from(""),
        salt: Some(salt),
        fill_from_rclone_config_error: None,
        create_repo_status: Status::Initial,
    });

    state.repo_creates.creates.insert(create_id, repo_create);

    create_id
}

pub fn create_loaded(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    load_mount_res: Result<String, remote::RemoteError>,
) {
    let no_existing_repos = state.repos.repos_by_id.is_empty();

    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return,
    };

    notify(store::Event::RepoCreate);

    let (create_load_status, primary_mount_id) = match load_mount_res {
        Ok(mount_id) => (Status::Loaded, Some(mount_id)),
        Err(remote::RemoteError::ApiError {
            code: remote::ApiErrorCode::NotFound,
            ..
        }) => (
            Status::Loading {
                loaded: form.create_load_status.loaded(),
            },
            None,
        ),
        Err(err) => (
            Status::Error {
                error: err,
                loaded: form.create_load_status.loaded(),
            },
            None,
        ),
    };

    form.create_load_status = create_load_status;
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

pub fn destroy(state: &mut store::State, notify: &store::Notify, create_id: u32) {
    notify(store::Event::RepoCreate);

    state.repo_creates.creates.remove(&create_id);
}

pub fn set_location(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    location: RemoteFilesLocation,
) {
    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return,
    };

    notify(store::Event::RepoCreate);

    form.location = Some(location);

    // hide the error when a new location is selected
    form.create_repo_status = Status::Initial
}

pub fn location_dir_picker_show(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    location_dir_picker_id: u32,
) {
    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return,
    };

    notify(store::Event::RepoCreate);

    form.location_dir_picker_id = Some(location_dir_picker_id);
}

pub fn location_dir_picker_cancel(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
) -> Option<u32> {
    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return None,
    };

    match form.location_dir_picker_id {
        Some(location_dir_picker_id) => {
            notify(store::Event::RepoCreate);

            form.location_dir_picker_id = None;

            Some(location_dir_picker_id)
        }
        None => None,
    }
}

pub fn set_password(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    password: String,
) {
    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return,
    };

    notify(store::Event::RepoCreate);

    form.password = password;
}

pub fn set_salt(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    salt: Option<String>,
) {
    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return,
    };

    notify(store::Event::RepoCreate);

    form.salt = salt;
}

pub fn fill_from_rclone_config(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    config: Result<rclone::config::Config, rclone::config::ParseConfigError>,
) {
    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return,
    };

    notify(store::Event::RepoCreate);

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

pub fn repo_creating(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
) -> Option<RepoCreateForm> {
    if !selectors::select_can_create(state, create_id) {
        return None;
    }

    let form = match state.repo_creates.creates.get_mut(&create_id) {
        Some(RepoCreate::Form(ref mut form)) => form,
        _ => return None,
    };

    notify(store::Event::RepoCreate);

    form.create_repo_status = Status::Loading {
        loaded: form.create_repo_status.loaded(),
    };

    Some(form.clone())
}

pub fn repo_created(
    state: &mut store::State,
    notify: &store::Notify,
    create_id: u32,
    res: Result<RepoCreated, CreateRepoError>,
) {
    notify(store::Event::RepoCreate);

    match res {
        Ok(created) => {
            state
                .repo_creates
                .creates
                .insert(create_id, RepoCreate::Created(created));
        }
        Err(err) => {
            match state.repo_creates.creates.get_mut(&create_id) {
                Some(RepoCreate::Form(ref mut form)) => {
                    form.create_repo_status = Status::Error {
                        error: err,
                        loaded: form.create_repo_status.loaded(),
                    };
                }
                _ => (),
            };
        }
    }
}
