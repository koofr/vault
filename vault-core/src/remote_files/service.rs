use std::sync::Arc;

use crate::{
    common::state::BoxAsyncRead,
    dialogs,
    remote::{
        models, remote::ListRecursiveItemStream, Remote, RemoteError, RemoteFileReader,
        RemoteFileUploadConflictResolution,
    },
    store,
    utils::path_utils,
};

use super::{errors::CreateDirError, mutations, selectors};

pub struct RemoteFilesService {
    remote: Arc<Remote>,
    dialogs_service: Arc<dialogs::DialogsService>,
    store: Arc<store::Store>,
}

impl RemoteFilesService {
    pub fn new(
        remote: Arc<Remote>,
        dialogs_service: Arc<dialogs::DialogsService>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            remote,
            dialogs_service,
            store,
        }
    }

    pub async fn load_places(&self) -> Result<(), RemoteError> {
        let mounts = self.remote.get_places().await?;

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RemoteFiles);

            mutations::places_loaded(state, mounts);
        });

        Ok(())
    }

    pub async fn load_bookmarks(&self) -> Result<(), RemoteError> {
        let bookmarks = self.remote.get_bookmarks().await?;

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RemoteFiles);

            mutations::bookmarks_loaded(state, bookmarks);
        });

        Ok(())
    }

    pub async fn load_shared(&self) -> Result<(), RemoteError> {
        let shared_files = self.remote.get_shared().await?;

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RemoteFiles);

            mutations::shared_files_loaded(state, shared_files);
        });

        Ok(())
    }

    pub async fn load_mount(&self, mount_id: &str) -> Result<String, RemoteError> {
        let mount = self.remote.get_mount(mount_id).await?;
        // mount_id parameter can be "primary" but we want an actual id
        let mount_id = mount.id.clone();

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RemoteFiles);

            mutations::mount_loaded(state, mount);
        });

        Ok(mount_id)
    }

    pub async fn load_files(&self, mount_id: &str, path: &str) -> Result<(), RemoteError> {
        let bundle = self.remote.get_bundle(mount_id, path).await?;

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::bundle_loaded(
                    state,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    path,
                    bundle,
                );
            });

        Ok(())
    }

    pub async fn load_file(&self, mount_id: &str, path: &str) -> Result<(), RemoteError> {
        let file = self.remote.get_file(mount_id, path).await?;

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::file_loaded(
                    state,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    path,
                    file,
                );
            });

        Ok(())
    }

    pub async fn get_file_reader(
        &self,
        mount_id: &str,
        path: &str,
    ) -> Result<RemoteFileReader, RemoteError> {
        self.remote.get_file_reader(&mount_id, &path).await
    }

    pub async fn get_list_recursive(
        &self,
        mount_id: &str,
        path: &str,
    ) -> Result<ListRecursiveItemStream, RemoteError> {
        self.remote.get_list_recursive(mount_id, path).await
    }

    pub async fn upload_file_reader(
        &self,
        mount_id: &str,
        parent_path: &str,
        name: &str,
        reader: BoxAsyncRead,
        size: Option<i64>,
        conflict_resolution: RemoteFileUploadConflictResolution,
        on_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
    ) -> Result<(String, models::FilesFile), RemoteError> {
        let file = self
            .remote
            .upload_file_reader(
                mount_id,
                parent_path,
                name,
                reader,
                size,
                conflict_resolution,
                on_progress,
            )
            .await?;

        let name = file.name.clone();
        let path = path_utils::join_path_name(parent_path, &name);

        self.file_created(mount_id, &path, file.clone());

        Ok((selectors::get_file_id(mount_id, &path), file))
    }

    pub async fn delete_file(&self, mount_id: &str, path: &str) -> Result<(), RemoteError> {
        self.remote.delete_file(mount_id, path).await?;

        self.file_removed(mount_id, path);

        Ok(())
    }

    pub async fn create_dir(
        &self,
        mount_id: &str,
        parent_path: &str,
    ) -> Result<(String, String), CreateDirError> {
        let input_value_validator_store = self.store.clone();
        let input_value_validator_mount_id = mount_id.to_owned();
        let input_value_validator_parent_path = parent_path.to_owned();

        let name = match self
            .dialogs_service
            .show(dialogs::state::DialogShowOptions {
                input_value_validator: Some(Box::new(move |value| {
                    input_value_validator_store
                        .with_state(|state| {
                            selectors::select_check_new_name_valid(
                                state,
                                &input_value_validator_mount_id,
                                &input_value_validator_parent_path,
                                value,
                            )
                        })
                        .is_ok()
                })),
                input_placeholder: Some(String::from("Folder name")),
                confirm_button_text: String::from("Create folder"),
                ..self
                    .dialogs_service
                    .build_prompt(String::from("Enter new folder name"))
            })
            .await
        {
            Some(name) => name,
            None => return Err(CreateDirError::Canceled),
        };

        let path = path_utils::join_path_name(&parent_path, &name);

        self.create_dir_name(mount_id, parent_path, &name).await?;

        Ok((name, path))
    }

    pub async fn create_dir_name(
        &self,
        mount_id: &str,
        parent_path: &str,
        name: &str,
    ) -> Result<(), RemoteError> {
        self.remote.create_dir(mount_id, parent_path, name).await?;

        let path = path_utils::join_path_name(parent_path, name);

        self.dir_created(mount_id, &path);

        Ok(())
    }

    pub async fn copy_file(
        &self,
        mount_id: &str,
        path: &str,
        to_mount_id: &str,
        to_path: &str,
    ) -> Result<(), RemoteError> {
        self.remote
            .copy_file(mount_id, path, to_mount_id, to_path)
            .await?;

        // self.file_copied() called from eventstream service

        Ok(())
    }

    pub async fn move_file(
        &self,
        mount_id: &str,
        path: &str,
        to_mount_id: &str,
        to_path: &str,
    ) -> Result<(), RemoteError> {
        self.remote
            .move_file(mount_id, path, to_mount_id, to_path)
            .await?;

        // self.file_moved() called from eventstream service

        Ok(())
    }

    pub async fn rename_file(
        &self,
        mount_id: &str,
        path: &str,
        new_name: &str,
    ) -> Result<(), RemoteError> {
        self.remote.rename_file(mount_id, path, new_name).await?;

        // self.file_moved() called from eventstream service

        Ok(())
    }

    pub fn file_created(&self, mount_id: &str, path: &str, file: models::FilesFile) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::file_created(
                    state,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    path,
                    file,
                );
            });
    }

    pub fn file_removed(&self, mount_id: &str, path: &str) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::file_removed(state, mutation_state, mutation_notify, mount_id, path);
            });
    }

    pub fn dir_created(&self, mount_id: &str, path: &str) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::dir_created(state, mutation_state, mutation_notify, mount_id, path);
            });
    }

    pub fn file_copied(&self, mount_id: &str, new_path: &str, file: models::FilesFile) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::file_copied(
                    state,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    new_path,
                    file,
                );
            });
    }

    pub fn file_moved(&self, mount_id: &str, path: &str, new_path: &str, file: models::FilesFile) {
        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                notify(store::Event::RemoteFiles);

                mutations::file_moved(
                    state,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    path,
                    new_path,
                    file,
                );
            });
    }
}
