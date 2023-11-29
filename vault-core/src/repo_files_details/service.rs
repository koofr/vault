use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use futures::{
    future::{self, BoxFuture},
    io::Cursor,
    stream::{AbortHandle, Abortable},
    AsyncReadExt, FutureExt,
};

use crate::{
    dialogs::{self, state::DialogShowOptions},
    http::HttpError,
    remote::{ApiErrorCode, RemoteError},
    remote_files::errors::RemoteFilesErrors,
    repo_files::{
        errors::{DeleteFileError, UploadFileReaderError},
        state::{RepoFile, RepoFilesUploadConflictResolution, RepoFilesUploadResult},
        RepoFilesService,
    },
    repo_files_read::{
        errors::GetFilesReaderError,
        state::{RepoFileReader, RepoFileReaderBuilder, RepoFileReaderProvider},
        RepoFilesReadService,
    },
    repos::ReposService,
    runtime, store,
    transfers::{downloadable::BoxDownloadable, errors::TransferError, TransfersService},
    types::{DecryptedName, EncryptedName, EncryptedPath, RepoId},
    user_error::UserError,
    utils::{on_end_reader::OnEndReader, repo_encrypted_path_utils},
};

use super::{
    errors::{LoadContentError, LoadDetailsError, SaveError},
    mutations, selectors,
    state::{RepoFilesDetailsContentData, RepoFilesDetailsOptions, SaveInitiator},
};

pub struct RepoFilesDetailsService {
    repos_service: Arc<ReposService>,
    repo_files_service: Arc<RepoFilesService>,
    repo_files_read_service: Arc<RepoFilesReadService>,
    dialogs_service: Arc<dialogs::DialogsService>,
    transfers_service: Arc<TransfersService>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,

    autosave_abort_handles: Arc<Mutex<HashMap<u32, AbortHandle>>>,
    repos_subscription_id: u32,
    mutation_subscription_id: u32,
}

impl RepoFilesDetailsService {
    pub fn new(
        repos_service: Arc<ReposService>,
        repo_files_service: Arc<RepoFilesService>,
        repo_files_read_service: Arc<RepoFilesReadService>,
        dialogs_service: Arc<dialogs::DialogsService>,
        transfers_service: Arc<TransfersService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Self {
        let repos_subscription_id = store.get_next_id();
        let repos_subscription_repo_files_service = repo_files_service.clone();
        let repos_subscription_store = store.clone();
        let repos_subscription_runtime = runtime.clone();

        store.on(
            repos_subscription_id,
            &[store::Event::Repos],
            Box::new(move |mutation_state, add_side_effect| {
                if !mutation_state.repos.unlocked_repos.is_empty() {
                    for details_id in repos_subscription_store.with_state(|state| {
                        selectors::select_unlocked_details(state, mutation_state)
                    }) {
                        let repo_files_service = repos_subscription_repo_files_service.clone();
                        let store = repos_subscription_store.clone();
                        let runtime = repos_subscription_runtime.clone();

                        add_side_effect(Box::new(move || {
                            // load errors are displayed inside details
                            runtime.spawn(
                                Self::load_file_inner(
                                    repo_files_service.clone(),
                                    store.clone(),
                                    details_id,
                                )
                                .map(|_| ())
                                .boxed(),
                            )
                        }))
                    }
                }
            }),
        );

        let mutation_subscription_id = store.get_next_id();

        store.mutation_on(
            mutation_subscription_id,
            &[store::MutationEvent::RepoFiles, store::MutationEvent::Repos],
            Box::new(move |state, notify, mutation_state, _| {
                mutations::handle_mutation(state, notify, mutation_state);
            }),
        );

        Self {
            repos_service,
            repo_files_service,
            repo_files_read_service,
            dialogs_service,
            transfers_service,
            store,
            runtime,

            autosave_abort_handles: Arc::new(Mutex::new(HashMap::new())),
            repos_subscription_id,
            mutation_subscription_id,
        }
    }

    pub fn create(
        self: Arc<Self>,
        repo_id: RepoId,
        path: &EncryptedPath,
        is_editing: bool,
        options: RepoFilesDetailsOptions,
    ) -> (u32, BoxFuture<'static, Result<(), LoadDetailsError>>) {
        let repo_files_subscription_id = self.store.get_next_id();

        let details_id = self.store.mutate(|state, notify, mutation_state, _| {
            mutations::create(
                state,
                notify,
                mutation_state,
                options,
                repo_id,
                path,
                is_editing,
                repo_files_subscription_id,
            )
        });

        let load_future: BoxFuture<'static, Result<(), LoadDetailsError>> = if self
            .store
            .with_state(|state| selectors::select_is_unlocked(state, details_id))
        {
            Self::load_file_inner(
                self.repo_files_service.clone(),
                self.store.clone(),
                details_id,
            )
            .boxed()
        } else {
            Box::pin(future::ready(Ok(())))
        };

        let repo_files_subscription_self = self.clone();

        self.store.on(
            repo_files_subscription_id,
            &[store::Event::RepoFiles, store::Event::RepoFilesDetails],
            Box::new(move |mutation_state, add_side_effect| {
                let (was_removed, should_reload) =
                    repo_files_subscription_self.store.with_state(|state| {
                        (
                            selectors::select_was_removed(state, mutation_state, details_id),
                            selectors::select_should_reload_content(
                                state,
                                mutation_state,
                                details_id,
                            ),
                        )
                    });

                if was_removed {
                    let side_effect_self = repo_files_subscription_self.clone();

                    add_side_effect(Box::new(move || {
                        side_effect_self.clone().runtime.spawn(Box::pin(async move {
                            side_effect_self.file_removed(details_id).await;
                        }));
                    }));
                }

                if should_reload {
                    let side_effect_self = repo_files_subscription_self.clone();

                    add_side_effect(Box::new(move || {
                        side_effect_self.clone().runtime.spawn(Box::pin(async move {
                            // errors will be stored in the store
                            let _ = side_effect_self.load_content(details_id).await;
                        }));
                    }))
                }
            }),
        );

        (details_id, load_future)
    }

    pub async fn destroy(self: Arc<Self>, details_id: u32) -> Result<(), SaveError> {
        loop {
            match self.clone().edit_cancel(details_id).await {
                Ok(()) => {}
                Err(err) => {
                    let message = format!("File could not be saved ({}). Do you want to Try again or Discard the changes?", err.user_error());

                    match self
                        .dialogs_service
                        .show(DialogShowOptions {
                            title: String::from("File could not be saved"),
                            message: Some(message),
                            confirm_button_text: String::from("Try again"),
                            cancel_button_text: Some(String::from("Discard changes")),
                            ..self.dialogs_service.build_confirm()
                        })
                        .await
                    {
                        Some(_) => continue,
                        None => {}
                    }
                }
            }

            let (repo_files_subscription_id, transfer_id) =
                self.store.mutate(|state, notify, mutation_state, _| {
                    mutations::destroy(state, notify, mutation_state, details_id)
                });

            if let Some(repo_files_subscription_id) = repo_files_subscription_id {
                self.store.remove_listener(repo_files_subscription_id);
            }

            if let Some(transfer_id) = transfer_id {
                self.transfers_service.clone().abort(transfer_id);
            }

            return Ok(());
        }
    }

    pub async fn load_file(&self, details_id: u32) -> Result<(), LoadDetailsError> {
        Self::load_file_inner(
            self.repo_files_service.clone(),
            self.store.clone(),
            details_id,
        )
        .await
    }

    pub async fn load_file_inner(
        repo_files_service: Arc<RepoFilesService>,
        store: Arc<store::Store>,
        details_id: u32,
    ) -> Result<(), LoadDetailsError> {
        if let Some((repo_id, path)) =
            store.with_state(|state| selectors::select_repo_id_path_owned(state, details_id))
        {
            store.mutate(|state, notify, _, _| {
                mutations::loading(state, notify, details_id);
            });

            let res = repo_files_service.load_files(&repo_id, &path).await;

            store.mutate(|state, notify, _, _| {
                mutations::loaded(
                    state,
                    notify,
                    details_id,
                    &repo_id,
                    &path,
                    res.as_ref().err(),
                );
            });

            res?;
        }

        Ok(())
    }

    pub async fn load_content(self: Arc<Self>, details_id: u32) -> Result<(), LoadContentError> {
        let file = self
            .store
            .mutate(|state, notify, _, _| mutations::content_loading(state, notify, details_id))?;

        let repo_id = file.repo_id.clone();
        let path = file.encrypted_path.clone();

        let res = match match self
            .repo_files_read_service
            .clone()
            .get_files_reader(vec![file])
        {
            Ok(provider) => provider.reader().await,
            Err(err) => Err(err),
        } {
            Ok(mut reader) => {
                let mut buf = Vec::new();

                match reader.reader.read_to_end(&mut buf).await {
                    Ok(_) => {
                        let remote_file = reader.remote_file.unwrap();

                        Ok(Some(RepoFilesDetailsContentData {
                            bytes: buf,
                            remote_size: remote_file.size,
                            remote_modified: remote_file.modified,
                            remote_hash: remote_file.hash,
                        }))
                    }
                    Err(err) => Err(TransferError::RemoteError(RemoteError::HttpError(
                        HttpError::ResponseError(err.to_string()),
                    ))),
                }
            }
            Err(err) => Err(err.into()),
        };

        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone().into());

        self.store.mutate(|state, notify, _, _| {
            mutations::content_loaded(state, notify, details_id, &repo_id, &path, res.into());
        });

        res_err
    }

    async fn get_file(self: Arc<Self>, details_id: u32) -> Result<RepoFile, GetFilesReaderError> {
        let file_store = self.store.clone();

        store::wait_for(
            self.store.clone(),
            &[store::Event::RepoFilesDetails],
            move |_| -> Option<Result<RepoFile, GetFilesReaderError>> {
                file_store.with_state(|state| selectors::select_file_reader_file(state, details_id))
            },
        )
        .await
    }

    pub async fn get_file_reader(
        self: Arc<Self>,
        details_id: u32,
    ) -> Result<RepoFileReaderProvider, GetFilesReaderError> {
        let file = self.clone().get_file(details_id).await?;

        let reader_builder_file = file.clone();
        let reader_builder_self = self.clone();

        Ok(self
            .repo_files_read_service
            .clone()
            .get_files_reader(vec![file])?
            .wrap_reader_builder(move |reader_builder| {
                let reader_builder_file: RepoFile = reader_builder_file.clone();
                let reader_builder_self = reader_builder_self.clone();

                Box::pin(async move {
                    reader_builder_self
                        .get_file_reader_wrap_reader_builder(
                            &reader_builder,
                            details_id,
                            reader_builder_file,
                        )
                        .await
                })
            }))
    }

    async fn get_file_reader_wrap_reader_builder(
        self: Arc<Self>,
        reader_builder: &RepoFileReaderBuilder,
        details_id: u32,
        file: RepoFile,
    ) -> Result<RepoFileReader, GetFilesReaderError> {
        let repo_id = file.repo_id.clone();
        let path = file.encrypted_path.clone();

        self.store.mutate(|state, notify, _, _| {
            mutations::file_reader_loading(state, notify, details_id, &file)
        })?;

        let reader = reader_builder().await?;

        let content_loaded_store = self.store.clone();

        Ok(reader.wrap_reader(|reader| {
            Box::pin(OnEndReader::new(
                reader,
                Box::new(move |res| {
                    content_loaded_store.mutate(|state, notify, _, _| {
                        mutations::content_loaded(
                            state,
                            notify,
                            details_id,
                            &repo_id,
                            &path,
                            res.map(|_| None).map_err(Into::into),
                        );
                    });
                }),
            ))
        }))
    }

    pub async fn download(
        self: Arc<Self>,
        details_id: u32,
        downloadable: BoxDownloadable,
    ) -> Result<(), TransferError> {
        let file = self.clone().get_file(details_id).await?;
        let repo_id = &file.repo_id;
        let path = file.encrypted_path.clone();

        let reader_provider = match self
            .repo_files_read_service
            .clone()
            .get_files_reader(vec![file.clone()])
        {
            Ok(reader_provider) => reader_provider,
            Err(err) => {
                let err = TransferError::from(err);

                self.store.mutate(|state, notify, _, _| {
                    mutations::content_error(state, notify, details_id, err.clone());
                });

                return Err(err);
            }
        };

        let (transfer_id, create_future) = self
            .transfers_service
            .clone()
            .download(reader_provider, downloadable);

        let future = match create_future.await {
            Ok(future) => future,
            Err(err) if matches!(err, TransferError::AlreadyExists) => {
                return Ok(());
            }
            Err(err) => {
                let err = TransferError::from(err);

                self.store.mutate(|state, notify, _, _| {
                    mutations::content_error(state, notify, details_id, err.clone());
                });

                return Err(err);
            }
        };

        if let Some(old_transfer_id) = self.store.mutate(|state, notify, _, _| {
            mutations::content_transfer_created(state, notify, details_id, &file, transfer_id)
        })? {
            self.transfers_service.clone().abort(old_transfer_id);
        }

        let res = future.await;

        self.store.mutate(|state, notify, _, _| {
            mutations::content_transfer_removed(
                state,
                notify,
                details_id,
                repo_id,
                &path,
                transfer_id,
                res,
            );
        });

        Ok(())
    }

    pub fn edit(&self, details_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::edit(state, notify, details_id);
        });
    }

    pub async fn edit_cancel(self: Arc<Self>, details_id: u32) -> Result<(), SaveError> {
        self.autosave_abort(details_id);

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
        let (repo_id, path, name, data, version, is_deleted) =
            self.clone().saving(details_id, &initiator).await?;

        let res = self
            .clone()
            .save_inner(initiator, repo_id, path, name, data, is_deleted)
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
    ) -> Result<
        (
            RepoId,
            EncryptedPath,
            EncryptedName,
            RepoFilesDetailsContentData,
            u32,
            bool,
        ),
        SaveError,
    > {
        let saving_store = self.store.clone();
        let initiator = initiator.to_owned();

        store::wait_for(
            self.store.clone(),
            &[store::Event::RepoFilesDetails],
            move |_| {
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
        repo_id: RepoId,
        path: EncryptedPath,
        original_name: EncryptedName,
        data: RepoFilesDetailsContentData,
        is_deleted: bool,
    ) -> Result<(EncryptedPath, RepoFilesUploadResult, bool), SaveError> {
        let mut autorename = false;

        let cipher = self.repos_service.get_cipher(&repo_id)?;

        let original_name = cipher.decrypt_filename(&original_name)?;

        let mut parent_path = repo_encrypted_path_utils::parent_path(&path)
            .ok_or_else(|| SaveError::CannotSaveRoot)?;

        if is_deleted {
            self.save_deleted_confirm_new_location(&initiator, &original_name)
                .await?;

            parent_path = EncryptedPath("/".into());
            autorename = true;
        }

        loop {
            let (name, conflict_resolution) = self
                .save_name_conflict_resolution(
                    &repo_id,
                    &parent_path,
                    &original_name,
                    autorename,
                    &data,
                )
                .await?;

            let encrypted_name = cipher.encrypt_filename(&name);
            let path = repo_encrypted_path_utils::join_path_name(&parent_path, &encrypted_name);

            let size = Some(data.bytes.len() as i64);
            let reader = Box::pin(Cursor::new(data.bytes.clone()));

            return match self
                .repo_files_service
                .clone()
                .upload_file_reader(
                    &repo_id,
                    &parent_path,
                    encrypted_name,
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
        name: &DecryptedName,
    ) -> Result<(), SaveError> {
        match &initiator {
            SaveInitiator::User => {
                let message = format!("File {} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location?", name.0);

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
            SaveInitiator::Autosave => {
                // we do not handle conflicts in autosave
                Err(SaveError::Canceled)
            }
            SaveInitiator::Cancel => {
                let message = format!("File {} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location or Discard the changes?", name.0);

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
        repo_id: &RepoId,
        parent_path: &EncryptedPath,
        name: &DecryptedName,
        autorename: bool,
        data: &RepoFilesDetailsContentData,
    ) -> Result<(DecryptedName, RepoFilesUploadConflictResolution), SaveError> {
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
                    if_remote_size: data.remote_size,
                    if_remote_modified: data.remote_modified,
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

    fn save_show_location_changed_alert(self: Arc<Self>, name: DecryptedName) {
        let location_changed_alert_self = self.clone();

        self.runtime.spawn(Box::pin(async move {
            let message = format!(
                "File {} was saved here because it could not be saved in its original location.",
                name.0
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
        match self
            .store
            .with_state(|state| selectors::select_repo_id_path_owned(state, details_id))
        {
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
            if selectors::select_is_not_deleting_or_deleted(state, details_id) {
                selectors::select_file_name(state, details_id)
            } else {
                None
            }
        }) {
            let message = format!("File {} is no longer accessible. Probably it was deleted or you no longer have access to it.", file_name.0);

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
        self.store.remove_listener(self.repos_subscription_id);
        self.store
            .mutation_remove_listener(self.mutation_subscription_id)
    }
}
