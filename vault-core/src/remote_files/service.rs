use std::{collections::HashMap, sync::Arc};

use crate::{
    common::state::BoxAsyncRead,
    dialogs,
    remote::{
        remote::{ListRecursiveItemStream, RemoteFileTagsSetConditions},
        Remote, RemoteError, RemoteFileUploadConflictResolution,
    },
    store,
    types::{MountId, RemoteFileId, RemoteName, RemotePath},
    utils::remote_path_utils,
};

use super::{
    errors::CreateDirError,
    mutations, selectors,
    state::{RemoteFile, RemoteFilesFileReader},
};

pub struct RemoteFilesService {
    remote: Arc<Remote>,
    dialogs_service: Arc<dialogs::DialogsService>,
    store: Arc<store::Store>,
    eventstream_events_mutation_subscription_id: u32,
}

impl RemoteFilesService {
    pub fn new(
        remote: Arc<Remote>,
        dialogs_service: Arc<dialogs::DialogsService>,
        store: Arc<store::Store>,
    ) -> Self {
        let eventstream_events_mutation_subscription_id = store.get_next_id();

        let remote_files_service = Self {
            remote,
            dialogs_service,
            store: store.clone(),
            eventstream_events_mutation_subscription_id,
        };

        store.mutation_on(
            eventstream_events_mutation_subscription_id,
            &[store::MutationEvent::EventstreamEvents],
            Box::new(move |state, notify, mutation_state, mutation_notify| {
                mutations::handle_eventstream_events_mutation(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                );
            }),
        );

        remote_files_service
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

    pub async fn load_mount(&self, mount_id: &MountId) -> Result<MountId, RemoteError> {
        let mount = self.remote.get_mount(mount_id).await?;
        // mount_id parameter can be "primary" but we want an actual id
        let mount_id = mount.id.clone();

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RemoteFiles);

            mutations::mount_loaded(state, mount);
        });

        Ok(mount_id)
    }

    pub async fn load_files(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<(), RemoteError> {
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

    pub async fn load_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<(), RemoteError> {
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
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<RemoteFilesFileReader, RemoteError> {
        let reader = self.remote.get_file_reader(&mount_id, &path).await?;

        Ok(RemoteFilesFileReader {
            file: mutations::files_file_to_remote_file(
                selectors::get_file_id(mount_id, &path.to_lowercase()),
                mount_id.to_owned(),
                path.to_owned(),
                reader.file,
            ),
            size: reader.size,
            reader: reader.reader,
        })
    }

    pub async fn get_list_recursive(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<ListRecursiveItemStream, RemoteError> {
        self.remote.get_list_recursive(mount_id, path).await
    }

    pub async fn upload_file_reader(
        &self,
        mount_id: &MountId,
        parent_path: &RemotePath,
        name: &RemoteName,
        reader: BoxAsyncRead,
        size: Option<i64>,
        conflict_resolution: RemoteFileUploadConflictResolution,
        on_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
    ) -> Result<(RemoteFileId, RemoteFile), RemoteError> {
        let file = self
            .remote
            .upload_file_reader(
                mount_id,
                parent_path,
                name,
                reader,
                size,
                None,
                conflict_resolution,
                on_progress,
            )
            .await?;

        let path = remote_path_utils::join_path_name(parent_path, &file.name);

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::file_created(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    &path,
                    file.clone(),
                );
            });

        let file_id = selectors::get_file_id(mount_id, &path.to_lowercase());

        let file = mutations::files_file_to_remote_file(
            file_id.clone(),
            mount_id.to_owned(),
            path.clone(),
            file,
        );

        Ok((file_id, file))
    }

    pub async fn delete_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<(), RemoteError> {
        self.remote
            .delete_file(mount_id, path, Default::default())
            .await?;

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::file_removed(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    path,
                );
            });

        Ok(())
    }

    pub async fn create_dir(
        &self,
        mount_id: &MountId,
        parent_path: &RemotePath,
    ) -> Result<(RemoteName, RemotePath), CreateDirError> {
        let input_value_validator_store = self.store.clone();
        let input_value_validator_mount_id = mount_id.to_owned();
        let input_value_validator_parent_path_lower = parent_path.to_lowercase();

        let name = match self
            .dialogs_service
            .show_validator(
                dialogs::state::DialogShowOptions {
                    input_placeholder: Some(String::from("Folder name")),
                    confirm_button_text: String::from("Create folder"),
                    ..self
                        .dialogs_service
                        .build_prompt(String::from("Enter new folder name"))
                },
                move |value| {
                    input_value_validator_store.with_state(|state| {
                        selectors::select_check_new_name_valid(
                            state,
                            &input_value_validator_mount_id,
                            &input_value_validator_parent_path_lower,
                            &RemoteName(value.clone()).to_lowercase(),
                        )
                        .map(|_| value)
                    })
                },
            )
            .await
        {
            Some(name) => RemoteName(name?),
            None => return Err(CreateDirError::Canceled),
        };

        let path = remote_path_utils::join_path_name(&parent_path, &name);

        self.create_dir_name(mount_id, parent_path, name.clone())
            .await?;

        Ok((name, path))
    }

    pub async fn create_dir_name(
        &self,
        mount_id: &MountId,
        parent_path: &RemotePath,
        name: RemoteName,
    ) -> Result<(), RemoteError> {
        let path = remote_path_utils::join_path_name(parent_path, &name);

        self.remote
            .create_dir(mount_id, parent_path, name.clone())
            .await?;

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::ensure_dir(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    &path,
                );
            });

        Ok(())
    }

    pub async fn copy_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        to_mount_id: &MountId,
        to_path: &RemotePath,
    ) -> Result<(), RemoteError> {
        self.remote
            .copy_file(mount_id, path, to_mount_id, to_path)
            .await?;

        // state is updated by eventstream event

        Ok(())
    }

    pub async fn move_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        to_mount_id: &MountId,
        to_path: &RemotePath,
    ) -> Result<(), RemoteError> {
        self.remote
            .move_file(mount_id, path, to_mount_id, to_path, Default::default())
            .await?;

        // state is updated by eventstream event

        Ok(())
    }

    pub async fn rename_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        new_name: RemoteName,
    ) -> Result<(), RemoteError> {
        self.remote.rename_file(mount_id, path, new_name).await?;

        // state is updated by eventstream event

        Ok(())
    }

    pub async fn set_tags(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        tags: HashMap<String, Vec<String>>,
        conditions: RemoteFileTagsSetConditions,
    ) -> Result<(), RemoteError> {
        self.remote
            .file_set_tags(mount_id, path, tags.clone(), conditions.clone())
            .await?;

        self.store
            .mutate(|state, notify, mutation_state, mutation_notify| {
                mutations::file_tags_set(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                    mount_id,
                    path,
                    tags,
                    &conditions,
                );
            });

        Ok(())
    }
}

impl Drop for RemoteFilesService {
    fn drop(&mut self) {
        self.store
            .mutation_remove_listener(self.eventstream_events_mutation_subscription_id);
    }
}
