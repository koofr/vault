use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{
    future::{self, BoxFuture},
    io::Cursor,
    stream::{AbortHandle, Abortable},
    AsyncReadExt,
};

use crate::{
    dialogs::{self, state::DialogShowOptions},
    eventstream::{self, service::MountSubscription},
    http::HttpError,
    remote::{ApiErrorCode, RemoteError},
    remote_files::errors::RemoteFilesErrors,
    repo_files::{
        errors::{DeleteFileError, LoadFilesError, UploadFileReaderError},
        state::{RepoFilesUploadConflictResolution, RepoFilesUploadResult},
        RepoFilesService,
    },
    repo_files_read::{errors::GetFilesReaderError, state::RepoFileReader, RepoFilesReadService},
    repos::selectors as repos_selectors,
    runtime, store,
    utils::path_utils::{self, normalize_path},
};

use super::{
    errors::{LoadContentError, LoadDetailsError, SaveError},
    mutations, selectors,
    state::{
        RepoFilesDetailsContentData, RepoFilesDetailsLocation, RepoFilesDetailsOptions,
        SaveInitiator,
    },
};

pub struct RepoFilesDetailsService {
    repo_files_service: Arc<RepoFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    eventstream_service: Arc<eventstream::EventStreamService>,
    dialogs_service: Arc<dialogs::DialogsService>,
    store: Arc<store::Store>,
    runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
    autosave_abort_handles: Arc<Mutex<HashMap<u32, AbortHandle>>>,
    repo_files_mutation_subscription_id: u32,
}

impl RepoFilesDetailsService {
    pub fn new(
        repo_files_service: Arc<RepoFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        eventstream_service: Arc<eventstream::EventStreamService>,
        dialogs_service: Arc<dialogs::DialogsService>,
        store: Arc<store::Store>,
        runtime: Arc<Box<dyn runtime::Runtime + Send + Sync>>,
    ) -> Arc<Self> {
        let repo_files_mutation_subscription_id = store.get_next_id();

        let repo_files_details_service = Arc::new(Self {
            repo_files_service,
            repo_files_read_service,
            eventstream_service,
            dialogs_service,
            store: store.clone(),
            runtime,
            autosave_abort_handles: Arc::new(Mutex::new(HashMap::new())),
            repo_files_mutation_subscription_id,
        });

        store.mutation_on(
            repo_files_mutation_subscription_id,
            &[store::MutationEvent::RepoFiles],
            Box::new(move |state, notify, mutation_state, _| {
                mutations::handle_repo_files_mutation(state, notify, mutation_state);
            }),
        );

        repo_files_details_service
    }

    pub fn create(
        self: Arc<Self>,
        repo_id: &str,
        path: &str,
        is_editing: bool,
        options: RepoFilesDetailsOptions,
    ) -> (u32, BoxFuture<'static, Result<(), LoadDetailsError>>) {
        let location = self.clone().get_location(repo_id, path, is_editing);

        let repo_files_subscription_id = self.store.get_next_id();

        let details_id = self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesDetails);

            mutations::create(state, options, location, repo_files_subscription_id)
        });

        let load_self = self.clone();

        let load_future: BoxFuture<'static, Result<(), LoadDetailsError>> = if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, details_id))
        {
            Box::pin(async move {
                load_self.load_file(details_id).await?;

                Ok(())
            })
        } else {
            Box::pin(future::ready(Ok(())))
        };

        let repo_files_subscription_self = self.clone();

        self.store.on(
            repo_files_subscription_id,
            &[store::Event::RepoFiles, store::Event::RepoFilesDetails],
            Box::new(move |mutation_state| {
                let (was_removed, should_reload) =
                    repo_files_subscription_self.store.with_state(|state| {
                        let was_removed =
                            selectors::select_was_removed(state, mutation_state, details_id);

                        let should_reload = !selectors::select_is_dirty(state, details_id)
                            && !selectors::select_is_saving(state, details_id)
                            && !selectors::select_is_content_loading(state, details_id)
                            && selectors::select_is_content_stale(state, details_id)
                            && !was_removed;

                        (was_removed, should_reload)
                    });

                if was_removed {
                    let file_removed_self = repo_files_subscription_self.clone();

                    repo_files_subscription_self
                        .runtime
                        .spawn(Box::pin(async move {
                            file_removed_self.file_removed(details_id).await;
                        }));
                }

                if should_reload {
                    let reload_content_self = repo_files_subscription_self.clone();

                    repo_files_subscription_self
                        .runtime
                        .spawn(Box::pin(async move {
                            // errors will be stored in the store
                            let _ = reload_content_self.load_content(details_id).await;
                        }));
                }
            }),
        );

        (details_id, load_future)
    }

    fn get_location(
        &self,
        repo_id: &str,
        path: &str,
        is_editing: bool,
    ) -> Result<RepoFilesDetailsLocation, LoadFilesError> {
        normalize_path(path)
            .map(|path| {
                let eventstream_mount_subscription =
                    self.clone().get_eventstream_mount_subscription(repo_id);

                mutations::create_location(
                    repo_id.to_owned(),
                    path,
                    eventstream_mount_subscription,
                    is_editing,
                )
            })
            .map_err(|_| LoadFilesError::RemoteError(RemoteFilesErrors::invalid_path()))
    }

    fn get_eventstream_mount_subscription(&self, repo_id: &str) -> Option<Arc<MountSubscription>> {
        self.store
            .with_state(|state| {
                repos_selectors::select_repo(state, repo_id)
                    .map(|repo| (repo.mount_id.clone(), repo.path.clone()))
            })
            .ok()
            .map(|(mount_id, mount_path)| {
                self.eventstream_service
                    .clone()
                    .get_mount_subscription(&mount_id, &mount_path)
            })
    }

    pub async fn destroy(self: Arc<Self>, details_id: u32) -> Result<(), SaveError> {
        self.clone().edit_cancel(details_id).await?;

        let repo_files_subscription_id = self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesDetails);

            mutations::destroy(state, details_id)
        });

        if let Some(repo_files_subscription_id) = repo_files_subscription_id {
            self.store.remove_listener(repo_files_subscription_id);
        }

        Ok(())
    }

    async fn load_file(&self, details_id: u32) -> Result<(), LoadFilesError> {
        if let Some((repo_id, path)) = self
            .store
            .with_state(|state| selectors::select_repo_id_path_owned(state, details_id))
        {
            let res = self.repo_files_service.load_files(&repo_id, &path).await;

            self.store.mutate(|state, notify, _, _| {
                notify(store::Event::RepoFilesDetails);

                mutations::loaded(state, details_id, &repo_id, &path, res.as_ref().err());
            });

            res?;
        }

        Ok(())
    }

    async fn load_content(self: Arc<Self>, details_id: u32) -> Result<(), LoadContentError> {
        let file = self
            .store
            .mutate(|state, notify, _, _| mutations::content_loading(state, notify, details_id))?;

        let repo_id = file.repo_id.clone();
        let path = file.path.decrypted_path()?.to_owned();

        let res = match self
            .repo_files_read_service
            .clone()
            .get_files_reader(&[file])
            .await
        {
            Ok(mut reader) => {
                let mut buf = Vec::new();

                match reader.reader.read_to_end(&mut buf).await {
                    Ok(_) => Ok((buf, reader.remote_file.unwrap())),
                    Err(err) => Err(GetFilesReaderError::RemoteError(RemoteError::HttpError(
                        HttpError::ResponseError(err.to_string()),
                    ))),
                }
            }
            Err(err) => Err(err),
        };

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone().into());

        self.store.mutate(|state, notify, _, _| {
            mutations::content_loaded(state, notify, details_id, repo_id, path, res);
        });

        res_err
    }

    pub async fn get_file_reader(
        self: Arc<Self>,
        details_id: u32,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let file = self
            .store
            .with_state(|state| selectors::select_file(state, details_id).map(|file| file.clone()))
            .ok_or(GetFilesReaderError::FileNotFound)?;

        self.repo_files_read_service
            .clone()
            .get_files_reader(&[file])
            .await
    }

    pub fn edit(&self, details_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::RepoFilesDetails);

            mutations::edit(state, details_id);
        });
    }

    pub async fn edit_cancel(self: Arc<Self>, details_id: u32) -> Result<(), SaveError> {
        self.autosave_abort(details_id);

        // TODO retry until ok or discarded
        let is_discarded = match self
            .clone()
            .save_if_dirty(details_id, SaveInitiator::Cancel)
            .await
        {
            Ok(()) => false,
            Err(SaveError::DiscardChanges { .. }) => true,
            Err(err) => return Err(err),
        };

        self.store.mutate(|state, notify, _, _| {
            mutations::edit_cancel(state, notify, details_id, is_discarded);
        });

        Ok(())
    }

    pub fn set_content(self: Arc<Self>, details_id: u32, content: Vec<u8>) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_content(state, notify, details_id, content);
        });

        self.autosave_schedule(details_id);
    }

    pub async fn save(self: Arc<Self>, details_id: u32) -> Result<(), SaveError> {
        self.clone()
            .save_initiator(details_id, SaveInitiator::User)
            .await
    }

    async fn save_initiator(
        self: Arc<Self>,
        details_id: u32,
        initiator: SaveInitiator,
    ) -> Result<(), SaveError> {
        let (repo_id, path, data, version, is_deleted) =
            self.clone().saving(details_id, &initiator).await?;

        let res = self
            .clone()
            .save_inner(initiator, repo_id, path, data, is_deleted)
            .await;

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store.mutate(|state, notify, _, _| {
            mutations::saved(state, notify, details_id, version, res);
        });

        res_err
    }

    async fn saving(
        self: Arc<Self>,
        details_id: u32,
        initiator: &SaveInitiator,
    ) -> Result<(String, String, RepoFilesDetailsContentData, u32, bool), SaveError> {
        let saving_store = self.store.clone();
        let initiator = initiator.to_owned();

        store::wait_for(
            self.store.clone(),
            &[store::Event::RepoFilesDetails],
            move || {
                saving_store.mutate(|state, notify, _, _| {
                    // wait for not saving
                    if selectors::select_is_saving(state, details_id) {
                        return None;
                    }

                    Some(mutations::saving(
                        state,
                        notify,
                        details_id,
                        initiator.clone(),
                    ))
                })
            },
        )
        .await
    }

    async fn save_inner(
        self: Arc<Self>,
        initiator: SaveInitiator,
        repo_id: String,
        path: String,
        data: RepoFilesDetailsContentData,
        is_deleted: bool,
    ) -> Result<(String, RepoFilesUploadResult, bool), SaveError> {
        let mut autorename = false;

        loop {
            let (mut parent_path, original_name) =
                path_utils::split_parent_name(&path).ok_or_else(|| SaveError::CannotSaveRoot)?;

            if is_deleted {
                self.save_deleted_confirm_new_location(&initiator, original_name)
                    .await?;

                parent_path = "/";
                autorename = true;
            }

            let (name, conflict_resolution) = self
                .save_name_conflict_resolution(
                    &repo_id,
                    parent_path,
                    original_name,
                    autorename,
                    &data,
                )
                .await?;

            let path = path_utils::join_path_name(&parent_path, &name);

            let size = Some(data.bytes.len() as i64);
            let reader = Box::pin(Cursor::new(data.bytes.clone()));

            return match self
                .repo_files_service
                .clone()
                .upload_file_reader(
                    &repo_id,
                    parent_path,
                    &name,
                    reader,
                    size,
                    conflict_resolution,
                    None,
                )
                .await
            {
                Ok(res) => {
                    if is_deleted {
                        self.save_show_location_changed_alert(original_name.to_owned());

                        Ok((path, res, true))
                    } else {
                        Ok((path, res, false))
                    }
                }
                Err(UploadFileReaderError::RemoteError(err))
                    if err.is_api_error_code(ApiErrorCode::Conflict) =>
                {
                    match self.save_handle_conflict(&initiator).await {
                        Ok(true) => {
                            autorename = true;

                            continue;
                        }
                        Ok(false) => Err(SaveError::RemoteError(err)),
                        Err(err) => Err(err),
                    }
                }
                Err(err) => Err(err.into()),
            };
        }
    }

    async fn save_deleted_confirm_new_location(
        &self,
        initiator: &SaveInitiator,
        name: &str,
    ) -> Result<(), SaveError> {
        match &initiator {
            SaveInitiator::User => {
                let message = format!("File {} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location?", name);

                match self
                    .dialogs_service
                    .show(DialogShowOptions {
                        title: String::from("File not accessible"),
                        message: Some(message),
                        confirm_button_text: String::from("Save to a new location"),
                        cancel_button_text: Some(String::from("Cancel")),
                        ..self.dialogs_service.build_confirm()
                    })
                    .await
                {
                    Some(_) => Ok(()),
                    None => Err(SaveError::Canceled),
                }
            }
            SaveInitiator::Autosave => panic!("unreachable"),
            SaveInitiator::Cancel => {
                let message = format!("File {} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location or Discard the changes?", name);

                match self
                    .dialogs_service
                    .show(DialogShowOptions {
                        title: String::from("File not accessible"),
                        message: Some(message),
                        confirm_button_text: String::from("Save to a new location"),
                        cancel_button_text: Some(String::from("Discard changes")),
                        ..self.dialogs_service.build_confirm()
                    })
                    .await
                {
                    Some(_) => Ok(()),
                    None => Err(SaveError::DiscardChanges {
                        should_destroy: true,
                    }),
                }
            }
        }
    }

    async fn save_name_conflict_resolution(
        &self,
        repo_id: &str,
        parent_path: &str,
        name: &str,
        autorename: bool,
        data: &RepoFilesDetailsContentData,
    ) -> Result<(String, RepoFilesUploadConflictResolution), SaveError> {
        Ok(if autorename {
            (
                self.repo_files_service
                    .get_unused_name(&repo_id, parent_path, name)
                    .await?,
                RepoFilesUploadConflictResolution::Error,
            )
        } else {
            (
                name.to_owned(),
                RepoFilesUploadConflictResolution::Overwrite {
                    if_remote_size: Some(data.remote_size),
                    if_remote_modified: Some(data.remote_modified),
                    if_remote_hash: data.remote_hash.clone(),
                },
            )
        })
    }

    async fn save_handle_conflict(&self, initiator: &SaveInitiator) -> Result<bool, SaveError> {
        match &initiator {
            SaveInitiator::User => {
                let message = String::from("Saving into the existing file is not possible. Do you want to Save your changes as a new file?");

                match self
                    .dialogs_service
                    .show(DialogShowOptions {
                        title: String::from(
                            "File was changed by someone else since your last save",
                        ),
                        message: Some(message),
                        confirm_button_text: String::from("Save as a new file"),
                        cancel_button_text: Some(String::from("Cancel")),
                        ..self.dialogs_service.build_confirm()
                    })
                    .await
                {
                    Some(_) => Ok(true),
                    None => Ok(false),
                }
            }
            SaveInitiator::Autosave => panic!("unreachable"),
            SaveInitiator::Cancel => {
                let message = String::from("Saving into the existing file is not possible. Do you want to Save your changes as a new file or Discard them?");

                match self
                    .dialogs_service
                    .show(DialogShowOptions {
                        title: String::from(
                            "File was changed by someone else since your last save",
                        ),
                        message: Some(message),
                        confirm_button_text: String::from("Save as a new file"),
                        cancel_button_text: Some(String::from("Discard changes")),
                        ..self.dialogs_service.build_confirm()
                    })
                    .await
                {
                    Some(_) => Ok(true),
                    None => Err(SaveError::DiscardChanges {
                        should_destroy: false,
                    }),
                }
            }
        }
    }

    fn save_show_location_changed_alert(self: Arc<Self>, name: String) {
        let location_changed_alert_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            let message = format!(
                "File {} was saved here because it could not be saved in its original location.",
                name
            );

            location_changed_alert_self
                .dialogs_service
                .show(DialogShowOptions {
                    message: Some(message),
                    ..location_changed_alert_self
                        .dialogs_service
                        .build_alert(String::from("File location changed"))
                })
                .await;
        }));
    }

    async fn save_if_dirty(
        self: Arc<Self>,
        details_id: u32,
        initiator: SaveInitiator,
    ) -> Result<(), SaveError> {
        match self.clone().save_initiator(details_id, initiator).await {
            Ok(()) => Ok(()),
            Err(SaveError::InvalidState) => Ok(()),
            Err(SaveError::NotDirty) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub async fn delete(&self, details_id: u32) -> Result<(), DeleteFileError> {
        match self.store.with_state(|state| {
            selectors::select_details_location(state, details_id)
                .map(|loc| (loc.repo_id.clone(), loc.path.clone()))
        }) {
            Some((repo_id, path)) => {
                let before_delete_store = self.store.clone();

                let res = self
                    .repo_files_service
                    .delete_files(
                        &[(repo_id, path)],
                        Some(Box::new(move || {
                            before_delete_store.mutate(|state, notify, _, _| {
                                mutations::deleting(state, notify, details_id)
                            });
                        })),
                    )
                    .await;

                self.store.mutate(|state, notify, _, _| {
                    mutations::deleted(state, notify, details_id, res.clone())
                });

                res
            }
            None => Err(DeleteFileError::RemoteError(RemoteFilesErrors::not_found())),
        }
    }

    async fn file_removed(&self, details_id: u32) {
        if let Some(file_name) = self.store.with_state(|state| {
            if selectors::select_is_not_deleting(state, details_id) {
                selectors::select_file_name(state, details_id).map(str::to_owned)
            } else {
                None
            }
        }) {
            let message = format!("File {} is no longer accessible. Probably it was deleted or you no longer have access to it.", file_name);

            self.dialogs_service
                .show(DialogShowOptions {
                    message: Some(message),
                    ..self
                        .dialogs_service
                        .build_alert(String::from("File not accessible"))
                })
                .await;
        }
    }

    fn autosave_schedule(self: Arc<Self>, details_id: u32) {
        let mut autosave_abort_handles = self.autosave_abort_handles.lock().unwrap();

        if autosave_abort_handles.contains_key(&details_id) {
            // autosave already scheduled
            return;
        }

        let autosave_interval = match self.store.with_state(|state| {
            selectors::select_details(state, details_id)
                .map(|details| details.options.autosave_interval)
        }) {
            Some(autosave_interval) => autosave_interval,
            None => return,
        };

        let (autosave_abort_handle, autosave_abort_registration) = AbortHandle::new_pair();

        autosave_abort_handles.insert(details_id, autosave_abort_handle);

        let autosave_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            let _ = Abortable::new(
                async move {
                    autosave_self.runtime.sleep(autosave_interval).await;

                    autosave_self
                        .autosave_abort_handles
                        .lock()
                        .unwrap()
                        .remove(&details_id);

                    // autosave is best effort
                    let _ = autosave_self
                        .save_if_dirty(details_id, SaveInitiator::Autosave)
                        .await;
                },
                autosave_abort_registration,
            )
            .await;
        }));
    }

    fn autosave_abort(&self, details_id: u32) {
        if let Some(abort_handle) = self
            .autosave_abort_handles
            .lock()
            .unwrap()
            .remove(&details_id)
        {
            abort_handle.abort();
        }
    }
}

impl Drop for RepoFilesDetailsService {
    fn drop(&mut self) {
        self.store
            .remove_listener(self.repo_files_mutation_subscription_id)
    }
}
