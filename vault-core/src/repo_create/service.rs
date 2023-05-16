use std::sync::Arc;

use crate::{
    cipher,
    cipher::random_password::random_password,
    common::state::Status,
    dir_pickers::selectors as dir_pickers_selectors,
    rclone,
    remote::{self, models, RemoteError},
    remote_files::{
        selectors as remote_files_selectors, state::RemoteFilesLocation, RemoteFilesService,
    },
    remote_files_dir_pickers::{self, RemoteFilesDirPickersService},
    repos::{
        mutations as repos_mutations, password_validator::generate_password_validator, ReposService,
    },
    store,
    utils::path_utils,
};

use super::{
    errors::RepoCreateError,
    mutations, selectors,
    state::{RepoCreateForm, RepoCreateState, RepoCreated},
};

const DEFAULT_DIR_NAMES: &'static [&'static str] = &[
    "My private documents",
    "My private pictures",
    "My private videos",
];

pub struct RepoCreateService {
    remote: Arc<remote::Remote>,
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    remote_files_dir_pickers_service: Arc<RemoteFilesDirPickersService>,
    store: Arc<store::Store>,
}

impl RepoCreateService {
    pub fn new(
        remote: Arc<remote::Remote>,
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        remote_files_dir_pickers_service: Arc<RemoteFilesDirPickersService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            remote,
            repos_service,
            remote_files_service,
            remote_files_dir_pickers_service,
            store,
        }
    }

    pub async fn init(&self) -> () {
        let salt = random_password(1024).unwrap();

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::init_loading(state, salt);
        });

        let (init_status, primary_mount_id) =
            match self.remote_files_service.load_mount("primary").await {
                Ok(mount_id) => (Status::Loaded, Some(mount_id)),
                Err(remote::RemoteError::ApiError {
                    code: remote::ApiErrorCode::NotFound,
                    ..
                }) => (Status::Loading, None),
                Err(err) => (Status::Error { error: err }, None),
            };

        // ignore the error
        let _ = self.repos_service.load_repos().await;

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::init_loaded(state, init_status, primary_mount_id);
        });
    }

    pub fn reset(&self) {
        self.location_dir_picker_cancel();

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::reset(state);
        });
    }

    pub fn set_location(&self, location: RemoteFilesLocation) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::set_location(state, location);
        });
    }

    pub fn set_password(&self, password: String) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::set_password(state, password);
        });
    }

    pub fn set_salt(&self, salt: Option<String>) {
        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::set_salt(state, salt);
        });
    }

    pub fn fill_from_rclone_config(&self, config: String) {
        let config = rclone::config::parse_config(&config);

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::fill_from_rclone_config(state, config);
        })
    }

    pub async fn location_dir_picker_show(&self) -> Result<(), RemoteError> {
        let (location, picker_id) = self.store.with_state(|state| {
            (
                selectors::select_location(state)
                    .cloned()
                    .or_else(|| selectors::select_primary_mount_location(state)),
                selectors::select_location_dir_picker_id(state),
            )
        });

        if picker_id.is_some() {
            return Ok(());
        }

        let location_dir_picker_id = self.remote_files_dir_pickers_service.create(
            remote_files_dir_pickers::state::Options {
                only_hosted_devices: true,
            },
        );

        self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::location_dir_picker_show(state, location_dir_picker_id);
        });

        if let Some(location) = &location {
            self.remote_files_dir_pickers_service
                .select_file(location_dir_picker_id, location)
                .await?;
        }

        self.remote_files_dir_pickers_service
            .load(location_dir_picker_id)
            .await?;

        // try to select it again if the data was not loaded yet
        if self.store.with_state(|state| {
            dir_pickers_selectors::select_selected_id(state, location_dir_picker_id).is_none()
        }) {
            if let Some(location) = &location {
                self.remote_files_dir_pickers_service
                    .select_file(location_dir_picker_id, location)
                    .await?;
            }
        }

        Ok(())
    }

    pub fn location_dir_picker_select(&self) {
        if let Some(location_file_id) = self.store.with_state(|state| {
            selectors::select_location_dir_picker_id(state)
                .and_then(|picker_id| {
                    dir_pickers_selectors::select_selected_file_id(state, picker_id)
                })
                .map(str::to_string)
        }) {
            if let Some(location) = self.store.with_state(|state| {
                remote_files_selectors::select_file(state, &location_file_id)
                    .map(|file| file.get_location())
            }) {
                self.set_location(location);
            }
        }

        self.location_dir_picker_cancel();
    }

    pub fn location_dir_picker_cancel(&self) {
        if let Some(location_dir_picker_id) = self.store.mutate(|state, notify| {
            notify(store::Event::RepoCreate);

            mutations::location_dir_picker_cancel(state)
        }) {
            self.remote_files_dir_pickers_service
                .destroy(location_dir_picker_id);
        }
    }

    pub fn location_dir_picker_check_create_dir(&self, name: &str) -> Result<(), RemoteError> {
        self.store
            .with_state(|state| selectors::select_location_dir_picker_check_create_dir(state, name))
    }

    pub async fn location_dir_picker_create_dir(
        &self,
        name: &str,
    ) -> Result<(), remote::RemoteError> {
        let picker_id = match self
            .store
            .with_state(|state| selectors::select_location_dir_picker_id(state))
        {
            Some(picker_id) => picker_id,
            None => return Ok(()),
        };

        self.remote_files_dir_pickers_service
            .create_dir(picker_id, name)
            .await
    }

    pub async fn create(&self) {
        self.location_dir_picker_cancel();

        match self.store.with_state(|state| {
            if !selectors::select_can_create(state) {
                return None;
            }

            match state.repo_create {
                Some(RepoCreateState::Form(ref form)) => Some(form.clone()),
                _ => None,
            }
        }) {
            Some(form) => {
                self.store.mutate(|state, notify| {
                    notify(store::Event::RepoCreate);

                    mutations::repo_creating(state);
                });

                let res = self.create_form(form).await;

                self.store.mutate(|state, notify| {
                    notify(store::Event::RepoCreate);

                    mutations::repo_create(state, res);
                });
            }
            None => return,
        };
    }

    async fn create_form(&self, form: RepoCreateForm) -> Result<RepoCreated, RepoCreateError> {
        let RepoCreateForm {
            location,
            password,
            salt,
            ..
        } = form;

        let location = location.unwrap();

        let already_exists = match (
            path_utils::parent_path(&location.path),
            path_utils::path_to_name(&location.path),
        ) {
            (Some(parent_path), Some(name)) => match self
                .remote
                .create_dir(&location.mount_id, parent_path, name)
                .await
            {
                Ok(_) => false,
                Err(remote::RemoteError::ApiError {
                    code: remote::ApiErrorCode::AlreadyExists,
                    ..
                }) => true,
                Err(err) => {
                    return Err(RepoCreateError::RemoteError(err));
                }
            },
            _ => false,
        };

        let cipher = cipher::Cipher::new(&password, salt.as_deref());

        let (password_validator, password_validator_encrypted) =
            generate_password_validator(&cipher).await;

        let repo = self
            .remote
            .create_vault_repo(models::VaultRepoCreate {
                mount_id: location.mount_id.clone(),
                path: location.path.clone(),
                salt: salt.clone(),
                password_validator,
                password_validator_encrypted,
            })
            .await?;
        let repo_id = repo.id.clone();

        if !already_exists {
            for name in DEFAULT_DIR_NAMES {
                let encrypted_name = cipher.encrypt_filename(name);

                self.remote_files_service
                    .create_dir(&location.mount_id, &location.path, &encrypted_name)
                    .await?;
            }
        }

        self.store.mutate(|state, notify| {
            notify(store::Event::Repos);

            repos_mutations::repo_loaded(state, repo);
        });

        let config = self
            .repos_service
            .get_repo_config(&repo_id, &password)
            .await
            .unwrap();

        Ok(RepoCreated { repo_id, config })
    }
}
