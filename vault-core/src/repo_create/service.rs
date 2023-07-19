use std::sync::Arc;

use futures::{future::BoxFuture, FutureExt};

use crate::{
    cipher::random_password::random_password,
    dir_pickers::selectors as dir_pickers_selectors,
    rclone, remote,
    remote_files::{
        errors::{CreateDirError, RemoteFilesErrors},
        selectors as remote_files_selectors,
        state::RemoteFilesLocation,
        RemoteFilesService,
    },
    remote_files_dir_pickers::{self, RemoteFilesDirPickersService},
    repos::ReposService,
    store,
};

use super::{mutations, selectors};

pub struct RepoCreateService {
    repos_service: Arc<ReposService>,
    remote_files_service: Arc<RemoteFilesService>,
    remote_files_dir_pickers_service: Arc<RemoteFilesDirPickersService>,
    store: Arc<store::Store>,
}

impl RepoCreateService {
    pub fn new(
        repos_service: Arc<ReposService>,
        remote_files_service: Arc<RemoteFilesService>,
        remote_files_dir_pickers_service: Arc<RemoteFilesDirPickersService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            repos_service,
            remote_files_service,
            remote_files_dir_pickers_service,
            store,
        }
    }

    pub fn create(self: Arc<Self>) -> (u32, BoxFuture<'static, Result<(), remote::RemoteError>>) {
        let salt = random_password(1024).unwrap();

        let create_id = self
            .store
            .mutate(|state, notify, _, _| mutations::create(state, notify, salt));

        let load_future = async move { self.clone().create_load(create_id).await }.boxed();

        (create_id, load_future)
    }

    async fn create_load(&self, create_id: u32) -> Result<(), remote::RemoteError> {
        let load_mount_res = self.remote_files_service.load_mount("primary").await;

        let load_mount_res_err = load_mount_res
            .as_ref()
            .map(|_| ())
            .map_err(|err| err.to_owned());

        // ignore the error
        let _ = self.repos_service.load_repos().await;

        self.store.mutate(|state, notify, _, _| {
            mutations::create_loaded(state, notify, create_id, load_mount_res);
        });

        load_mount_res_err
    }

    pub fn set_location(&self, create_id: u32, location: RemoteFilesLocation) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_location(state, notify, create_id, location);
        });
    }

    pub fn set_password(&self, create_id: u32, password: String) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_password(state, notify, create_id, password);
        });
    }

    pub fn set_salt(&self, create_id: u32, salt: Option<String>) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_salt(state, notify, create_id, salt);
        });
    }

    pub fn fill_from_rclone_config(
        &self,
        create_id: u32,
        config: String,
    ) -> Result<(), rclone::config::ParseConfigError> {
        let res = rclone::config::parse_config(&config);

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store.mutate(|state, notify, _, _| {
            mutations::fill_from_rclone_config(state, notify, create_id, res);
        });

        res_err
    }

    pub async fn location_dir_picker_show(
        &self,
        create_id: u32,
    ) -> Result<(), remote::RemoteError> {
        let (location, picker_id) = self.store.with_state(|state| {
            (
                selectors::select_location(state, create_id)
                    .cloned()
                    .or_else(|| selectors::select_primary_mount_location(state, create_id)),
                selectors::select_location_dir_picker_id(state, create_id),
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

        self.store.mutate(|state, notify, _, _| {
            mutations::location_dir_picker_show(state, notify, create_id, location_dir_picker_id);
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

    pub async fn location_dir_picker_click(
        &self,
        create_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) -> Result<(), remote::RemoteError> {
        let picker_id = self
            .store
            .with_state(|state| selectors::select_location_dir_picker_id(state, create_id))
            .ok_or_else(RemoteFilesErrors::not_found)?;

        self.remote_files_dir_pickers_service
            .click(picker_id, item_id, is_arrow)
            .await
    }

    pub fn location_dir_picker_select(&self, create_id: u32) {
        if let Some(location_file_id) = self.store.with_state(|state| {
            selectors::select_location_dir_picker_id(state, create_id)
                .and_then(|picker_id| {
                    dir_pickers_selectors::select_selected_file_id(state, picker_id)
                })
                .map(str::to_string)
        }) {
            if let Some(location) = self.store.with_state(|state| {
                remote_files_selectors::select_file(state, &location_file_id)
                    .map(|file| file.get_location())
            }) {
                self.set_location(create_id, location);
            }
        }

        self.location_dir_picker_cancel(create_id);
    }

    pub fn location_dir_picker_cancel(&self, create_id: u32) {
        if let Some(location_dir_picker_id) = self.store.mutate(|state, notify, _, _| {
            mutations::location_dir_picker_cancel(state, notify, create_id)
        }) {
            self.remote_files_dir_pickers_service
                .destroy(location_dir_picker_id);
        }
    }

    pub async fn location_dir_picker_create_dir(
        &self,
        create_id: u32,
    ) -> Result<(), CreateDirError> {
        let (mount_id, parent_path, picker_id) = self
            .store
            .with_state(|state| {
                let picker_id = selectors::select_location_dir_picker_id(state, create_id)?;

                let remote_file =
                    remote_files_dir_pickers::selectors::select_selected_file(state, picker_id)?;

                Some((
                    remote_file.mount_id.to_owned(),
                    remote_file.path.to_owned(),
                    picker_id,
                ))
            })
            .ok_or_else(RemoteFilesErrors::not_found)?;

        let (_, path) = self
            .remote_files_service
            .create_dir(&mount_id, &parent_path)
            .await?;

        self.remote_files_dir_pickers_service
            .select_file(picker_id, &RemoteFilesLocation { mount_id, path })
            .await?;

        Ok(())
    }

    pub async fn create_repo(&self, create_id: u32) {
        self.location_dir_picker_cancel(create_id);

        let form = match self
            .store
            .mutate(|state, notify, _, _| mutations::repo_creating(state, notify, create_id))
        {
            Some(form) => form,
            None => return,
        };

        let location = form.location.unwrap();

        let res = self
            .repos_service
            .create_repo(
                &location.mount_id,
                &location.path,
                &form.password,
                form.salt.as_deref(),
            )
            .await;

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoCreate);

            mutations::repo_created(state, notify, create_id, res);
        });
    }

    pub fn destroy(&self, create_id: u32) {
        self.location_dir_picker_cancel(create_id);

        self.store.mutate(|state, notify, _, _| {
            mutations::destroy(state, notify, create_id);
        });
    }
}
