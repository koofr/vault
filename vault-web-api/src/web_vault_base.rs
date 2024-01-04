use std::{
    collections::{hash_map, HashMap},
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{future::BoxFuture, FutureExt};

use vault_core::{
    common, dialogs,
    dir_pickers::state::DirPickerItemId,
    files, notifications, oauth2, remote_files, repo_config_backup, repo_create, repo_files,
    repo_files_browsers, repo_files_details, repo_files_move, repo_remove, repo_space_usage,
    repo_unlock, repos,
    store::{self, Event, Subscription},
    transfers,
    types::{DecryptedName, EncryptedPath, RepoFileId, RepoId, TimeMillis},
    user_error,
    user_error::UserError,
    Vault,
};

use crate::{dto, web_errors::WebErrors};

pub type Callback = Box<dyn Fn() + Send + Sync>;

pub type Data<T> = Arc<Mutex<HashMap<u32, T>>>;

#[derive(Default)]
pub struct SubscriptionData {
    pub notifications: Data<Vec<dto::Notification>>,
    pub dialogs: Data<Vec<u32>>,
    pub dialog: Data<Option<dto::Dialog>>,
    pub oauth2_status: Data<dto::Status>,
    pub user: Data<Option<dto::User>>,
    pub user_profile_picture_loaded: Data<bool>,
    pub repos: Data<dto::Repos>,
    pub repos_repo: Data<dto::RepoInfo>,
    pub repo_create_info: Data<Option<dto::RepoCreateInfo>>,
    pub repo_unlock_info: Data<Option<dto::RepoUnlockInfo>>,
    pub repo_remove_info: Data<Option<dto::RepoRemoveInfo>>,
    pub repo_config_backup_info: Data<Option<dto::RepoConfigBackupInfo>>,
    pub repo_space_usage_info: Data<Option<dto::RepoSpaceUsageInfo>>,
    pub repo_files_file: Data<Option<dto::RepoFile>>,
    pub transfers_is_active: Data<bool>,
    pub transfers_summary: Data<dto::TransfersSummary>,
    pub transfers_list: Data<dto::TransfersList>,
    pub dir_pickers_items: Data<Vec<dto::DirPickerItem>>,
    pub repo_files_browsers_info: Data<Option<dto::RepoFilesBrowserInfo>>,
    pub repo_files_details_info: Data<Option<dto::RepoFilesDetailsInfo>>,
    pub repo_files_details_file: Data<Option<dto::RepoFile>>,
    pub repo_files_details_content_bytes: Data<dto::Versioned<Option<Vec<u8>>>>,
    pub repo_files_move_info: Data<Option<dto::RepoFilesMoveInfo>>,
    pub space_usage: Data<Option<dto::SpaceUsage>>,
}

pub struct WebVaultBase {
    pub vault: Arc<Vault>,
    pub errors: Arc<WebErrors>,
    pub subscription_data: SubscriptionData,
    pub subscription: Subscription,
    pub file_icon_factory: vault_file_icon::FileIconFactory,
}

impl WebVaultBase {
    pub fn new(vault: Arc<Vault>) -> Self {
        let errors = Arc::new(WebErrors::new(vault.clone()));

        let subscription_data = SubscriptionData::default();
        let subscription = Subscription::new(vault.store.clone());

        let file_icon_theme = vault_file_icon::FileIconTheme::default();
        let file_icon_factory = vault_file_icon::FileIconFactory::new(&file_icon_theme);

        Self {
            vault,
            errors,
            subscription_data,
            subscription,
            file_icon_factory,
        }
    }

    // errors

    pub fn handle_error(&self, user_error: impl user_error::UserError) {
        self.errors.handle_error(user_error);
    }

    pub fn handle_result(&self, result: Result<(), impl user_error::UserError>) {
        self.errors.handle_result(result);
    }

    // spawn

    pub fn spawn<F>(&self, f: F)
    where
        F: FnOnce(Arc<Vault>) -> BoxFuture<'static, ()>,
    {
        self.vault.runtime.spawn(f(self.vault.clone()));
    }

    pub fn spawn_result<F, E>(&self, f: F)
    where
        F: FnOnce(Arc<Vault>) -> BoxFuture<'static, Result<(), E>> + Send + 'static,
        E: UserError + 'static,
    {
        let errors = self.errors.clone();

        self.spawn(move |vault| {
            let errors = errors.clone();

            async move { errors.handle_result(f(vault.clone()).await) }.boxed()
        });
    }

    // subscription

    pub fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[Event],
        callback: Callback,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<Vault>) -> T + Send + Sync + 'static,
    ) -> u32 {
        let vault = self.vault.clone();

        self.subscription
            .subscribe(events, callback, subscription_data, move || {
                generate_data(vault.clone())
            })
    }

    pub fn subscribe_changed<T: Clone + Send + 'static>(
        &self,
        events: &[Event],
        callback: Callback,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<Vault>, hash_map::Entry<'_, u32, T>) -> bool + Send + Sync + 'static,
    ) -> u32 {
        let vault = self.vault.clone();

        self.subscription
            .subscribe_changed(events, callback, subscription_data, move |entry| {
                generate_data(vault.clone(), entry)
            })
    }

    pub fn get_data<T: Clone + Send>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Option<T> {
        self.subscription.get_data(id, subscription_data)
    }

    pub fn unsubscribe(&self, id: u32) {
        self.subscription.unsubscribe(id);
    }

    // lifecycle

    pub fn load(&self) {
        self.spawn_result(|vault| {
            async move {
                match vault.load() {
                    Ok(load_future) => load_future.await,
                    Err(err) => Err(err),
                }
            }
            .boxed()
        });
    }

    pub fn logout(&self) {
        self.handle_result(self.vault.logout());
    }

    pub fn app_visible(&self) {
        self.vault.app_visible();
    }

    pub fn app_hidden(&self) {
        self.vault.app_hidden();
    }

    // relative_time

    pub fn relative_time(&self, value: f64, with_modifier: bool) -> dto::RelativeTime {
        dto::RelativeTime::from(
            self.vault
                .relative_time(TimeMillis(value as i64), with_modifier),
        )
    }

    // notifications

    pub fn notifications_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Notifications],
            cb,
            self.subscription_data.notifications.clone(),
            move |vault| {
                vault.with_state(|state| {
                    notifications::selectors::select_notifications(state)
                        .into_iter()
                        .map(Into::into)
                        .collect()
                })
            },
        )
    }

    pub fn notifications_data(&self, id: u32) -> Option<Vec<dto::Notification>> {
        self.get_data(id, self.subscription_data.notifications.clone())
    }

    pub fn notifications_remove(&self, notification_id: u32) {
        self.vault.notifications_remove(notification_id);
    }

    pub fn notifications_remove_after(&self, notification_id: u32, duration_ms: u32) {
        self.spawn(move |vault| {
            async move {
                vault
                    .notifications_remove_after(
                        notification_id,
                        Duration::from_millis(duration_ms as u64),
                    )
                    .await
            }
            .boxed()
        });
    }

    pub fn notifications_remove_all(&self) {
        self.vault.notifications_remove_all();
    }

    // dialogs

    pub fn dialogs_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Dialogs],
            cb,
            self.subscription_data.dialogs.clone(),
            move |vault| {
                vault.with_state(|state| {
                    dialogs::selectors::select_dialogs(state)
                        .iter()
                        .map(|dialog| dialog.id)
                        .collect()
                })
            },
        )
    }

    pub fn dialogs_data(&self, id: u32) -> Option<Vec<u32>> {
        self.get_data(id, self.subscription_data.dialogs.clone())
    }

    pub fn dialogs_dialog_subscribe(&self, dialog_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Dialogs],
            cb,
            self.subscription_data.dialog.clone(),
            move |vault| {
                vault.with_state(|state| {
                    dialogs::selectors::select_dialog_info(state, dialog_id).map(Into::into)
                })
            },
        )
    }

    pub fn dialogs_dialog_data(&self, id: u32) -> Option<dto::Dialog> {
        self.get_data(id, self.subscription_data.dialog.clone())
            .flatten()
    }

    pub fn dialogs_confirm(&self, dialog_id: u32) {
        self.vault.dialogs_confirm(dialog_id);
    }

    pub fn dialogs_cancel(&self, dialog_id: u32) {
        self.vault.dialogs_cancel(dialog_id);
    }

    pub fn dialogs_set_input_value(&self, dialog_id: u32, value: String) {
        self.vault.dialogs_set_input_value(dialog_id, value);
    }

    // oauth2

    pub fn oauth2_status_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Auth],
            cb,
            self.subscription_data.oauth2_status.clone(),
            move |vault| vault.with_state(|state| oauth2::selectors::select_status(state).into()),
        )
    }

    pub fn oauth2_status_data(&self, id: u32) -> Option<dto::Status> {
        self.get_data(id, self.subscription_data.oauth2_status.clone())
    }

    pub fn oauth2_start_login_flow(&self) -> Option<String> {
        match self.vault.oauth2_start_login_flow() {
            Ok(url) => Some(url),
            Err(err) => {
                self.handle_error(err);
                None
            }
        }
    }

    pub fn oauth2_start_logout_flow(&self) -> Option<String> {
        match self.vault.oauth2_start_logout_flow() {
            Ok(url) => Some(url),
            Err(err) => {
                self.handle_error(err);
                None
            }
        }
    }

    pub async fn oauth2_finish_flow_url(&self, url: String) -> bool {
        let res = self.vault.oauth2_finish_flow_url(&url).await;

        let success = res.is_ok();

        self.handle_result(res);

        success
    }

    // config

    pub fn config_get_base_url(&self) -> String {
        self.vault.with_state(|state| state.config.base_url.clone())
    }

    // user

    pub fn user_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::User],
            cb,
            self.subscription_data.user.clone(),
            move |vault| vault.with_state(|state| state.user.user.as_ref().map(Into::into)),
        )
    }

    pub fn user_data(&self, id: u32) -> Option<dto::User> {
        self.get_data(id, self.subscription_data.user.clone())
            .flatten()
    }

    pub fn user_profile_picture_loaded_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::User],
            cb,
            self.subscription_data.user_profile_picture_loaded.clone(),
            move |vault| {
                vault.with_state(|state| {
                    state
                        .user
                        .user
                        .as_ref()
                        .map(|user| match &user.profile_picture_status {
                            common::state::Status::Loaded => true,
                            _ => false,
                        })
                        .unwrap_or(false)
                })
            },
        )
    }

    pub fn user_profile_picture_loaded_data(&self, id: u32) -> bool {
        self.get_data(
            id,
            self.subscription_data.user_profile_picture_loaded.clone(),
        )
        .unwrap_or(false)
    }

    pub fn user_get_profile_picture(&self) -> Option<Vec<u8>> {
        self.vault.with_state(|state| {
            state
                .user
                .user
                .as_ref()
                .and_then(|user| user.profile_picture_bytes.clone())
        })
    }

    pub fn user_ensure_profile_picture(&self) {
        self.spawn_result(move |vault| {
            async move { vault.user_ensure_profile_picture().await }.boxed()
        });
    }

    // file_icon

    pub fn file_icon_svg(&self, props: dto::FileIconProps) -> String {
        let (svg, _, _) = self.file_icon_factory.generate_svg(&props.into());
        svg
    }

    // repos

    pub fn repos_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Repos],
            cb,
            self.subscription_data.repos.clone(),
            move |vault| vault.with_state(|state| dto::Repos::from(state)),
        )
    }

    pub fn repos_data(&self, id: u32) -> Option<dto::Repos> {
        self.get_data(id, self.subscription_data.repos.clone())
    }

    pub fn repos_repo_subscribe(&self, repo_id: String, cb: Callback) -> u32 {
        let repo_id = RepoId(repo_id);

        self.subscribe(
            &[Event::Repos],
            cb,
            self.subscription_data.repos_repo.clone(),
            move |vault| {
                vault.with_state(|state| {
                    (&repos::selectors::select_repo_info(state, &repo_id)).into()
                })
            },
        )
    }

    pub fn repos_repo_data(&self, id: u32) -> Option<dto::RepoInfo> {
        self.get_data(id, self.subscription_data.repos_repo.clone())
    }

    pub fn repos_lock_repo(&self, repo_id: String) {
        self.handle_result(
            self.vault
                .repos_lock_repo(&RepoId(repo_id))
                .or_else(|err| match err {
                    // ignore already locked
                    repos::errors::LockRepoError::RepoLocked(_) => Ok(()),
                    _ => Err(err),
                }),
        );
    }

    pub fn repos_touch_repo(&self, repo_id: String) {
        self.handle_result(self.vault.repos_touch_repo(&RepoId(repo_id)));
    }

    pub fn repos_set_auto_lock(&self, repo_id: String, auto_lock: dto::RepoAutoLock) {
        self.handle_result(
            self.vault
                .repos_set_auto_lock(&RepoId(repo_id), auto_lock.into()),
        );
    }

    pub fn repos_set_default_auto_lock(&self, auto_lock: dto::RepoAutoLock) {
        self.vault.repos_set_default_auto_lock(auto_lock.into());
    }

    // repo_create

    pub fn repo_create_create(&self) -> u32 {
        let (create_id, create_load_future) = self.vault.repo_create_create();

        self.vault.runtime.spawn(
            async move {
                // error is displayed in the details component
                let _ = create_load_future.await;
            }
            .boxed(),
        );

        create_id
    }

    pub fn repo_create_info_subscribe(&self, create_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoCreate, Event::DirPickers],
            cb,
            self.subscription_data.repo_create_info.clone(),
            move |vault| {
                use remote_files::selectors as remote_files_selectors;
                use repo_create::{selectors, state::RepoCreate};

                vault.with_state(|state| {
                    selectors::select_repo_create(state, create_id).map(|repo_create| {
                        match repo_create {
                            repo_create::state::RepoCreate::Form(form) => {
                                let location_breadcrumbs = form
                                    .location
                                    .as_ref()
                                    .map(|location| {
                                        remote_files_selectors::select_breadcrumbs(
                                            state,
                                            &location.mount_id,
                                            &location.path,
                                        )
                                    })
                                    .unwrap_or(Vec::new());

                                dto::RepoCreateInfo::Form(dto::RepoCreateForm {
                                    create_load_status: (&form.create_load_status).into(),
                                    location: form
                                        .location
                                        .as_ref()
                                        .map(|location| location.into()),
                                    location_breadcrumbs: location_breadcrumbs
                                        .iter()
                                        .map(dto::RemoteFilesBreadcrumb::from)
                                        .collect(),
                                    location_dir_picker_id: form.location_dir_picker_id,
                                    location_dir_picker_can_select:
                                        selectors::select_location_dir_picker_can_select(
                                            state, create_id,
                                        ),
                                    location_dir_picker_create_dir_enabled:
                                        selectors::select_location_dir_picker_create_dir_enabled(
                                            state, create_id,
                                        ),
                                    password: form.password.clone(),
                                    salt: form.salt.clone(),
                                    fill_from_rclone_config_error: form
                                        .fill_from_rclone_config_error
                                        .as_ref()
                                        .map(|e| e.to_string()),
                                    can_create: selectors::select_can_create(state, create_id),
                                    create_repo_status: (&form.create_repo_status).into(),
                                })
                            }
                            RepoCreate::Created(created) => {
                                dto::RepoCreateInfo::Created(created.into())
                            }
                        }
                    })
                })
            },
        )
    }

    pub fn repo_create_info_data(&self, id: u32) -> Option<dto::RepoCreateInfo> {
        self.get_data(id, self.subscription_data.repo_create_info.clone())
            .flatten()
    }

    pub fn repo_create_set_password(&self, create_id: u32, password: String) {
        self.vault.repo_create_set_password(create_id, password);
    }

    pub fn repo_create_set_salt(&self, create_id: u32, salt: Option<String>) {
        self.vault.repo_create_set_salt(create_id, salt);
    }

    pub fn repo_create_fill_from_rclone_config(&self, create_id: u32, config: String) {
        let _ = self
            .vault
            .repo_create_fill_from_rclone_config(create_id, config);
    }

    pub fn repo_create_location_dir_picker_show(&self, create_id: u32) {
        self.spawn_result(move |vault| {
            async move { vault.repo_create_location_dir_picker_show(create_id).await }.boxed()
        });
    }

    pub fn repo_create_location_dir_picker_click(
        &self,
        create_id: u32,
        item_id: String,
        is_arrow: bool,
    ) {
        self.spawn_result(move |vault| {
            async move {
                vault
                    .repo_create_location_dir_picker_click(
                        create_id,
                        &DirPickerItemId(item_id),
                        is_arrow,
                    )
                    .await
            }
            .boxed()
        });
    }

    pub fn repo_create_location_dir_picker_select(&self, create_id: u32) {
        self.vault.repo_create_location_dir_picker_select(create_id);
    }

    pub fn repo_create_location_dir_picker_cancel(&self, create_id: u32) {
        self.vault.repo_create_location_dir_picker_cancel(create_id);
    }

    pub fn repo_create_location_dir_picker_create_dir(&self, create_id: u32) {
        self.spawn_result(move |vault| {
            async move {
                match vault
                    .repo_create_location_dir_picker_create_dir(create_id)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(remote_files::errors::CreateDirError::Canceled) => Ok(()),
                    Err(err) => Err(err),
                }
            }
            .boxed()
        });
    }

    pub fn repo_create_create_repo(&self, create_id: u32) {
        self.spawn(move |vault| {
            async move { vault.repo_create_create_repo(create_id).await }.boxed()
        });
    }

    pub fn repo_create_destroy(&self, create_id: u32) {
        self.vault.repo_create_destroy(create_id);
    }

    // repo_unlock

    pub fn repo_unlock_create(&self, repo_id: String, options: dto::RepoUnlockOptions) -> u32 {
        self.vault
            .repo_unlock_create(RepoId(repo_id), options.into())
    }

    pub fn repo_unlock_info_subscribe(&self, unlock_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoUnlock],
            cb,
            self.subscription_data.repo_unlock_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_unlock::selectors::select_info(state, unlock_id).map(|info| {
                        dto::RepoUnlockInfo {
                            status: info.status.into(),
                            repo_name: info.repo_name.map(|x| x.0.clone()),
                        }
                    })
                })
            },
        )
    }

    pub fn repo_unlock_info_data(&self, id: u32) -> Option<dto::RepoUnlockInfo> {
        self.get_data(id, self.subscription_data.repo_unlock_info.clone())
            .flatten()
    }

    pub fn repo_unlock_unlock(&self, unlock_id: u32, password: String) {
        let _ = self.vault.repo_unlock_unlock(unlock_id, &password);
    }

    pub fn repo_unlock_destroy(&self, unlock_id: u32) {
        self.vault.repo_unlock_destroy(unlock_id);
    }

    // repo_remove

    pub fn repo_remove_create(&self, repo_id: String) -> u32 {
        self.vault.repo_remove_create(RepoId(repo_id))
    }

    pub fn repo_remove_info_subscribe(&self, remove_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoRemove],
            cb,
            self.subscription_data.repo_remove_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_remove::selectors::select_info(state, remove_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_remove_info_data(&self, id: u32) -> Option<dto::RepoRemoveInfo> {
        self.get_data(id, self.subscription_data.repo_remove_info.clone())
            .flatten()
    }

    pub async fn repo_remove_remove(&self, remove_id: u32, password: String) -> bool {
        self.vault
            .repo_remove_remove(remove_id, &password)
            .await
            .is_ok()
    }

    pub fn repo_remove_destroy(&self, remove_id: u32) {
        self.vault.repo_remove_destroy(remove_id);
    }

    // repo_config_backup

    pub fn repo_config_backup_create(&self, repo_id: String) -> u32 {
        self.vault.repo_config_backup_create(RepoId(repo_id))
    }

    pub fn repo_config_backup_info_subscribe(&self, backup_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoConfigBackup],
            cb,
            self.subscription_data.repo_config_backup_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_config_backup::selectors::select_info(state, backup_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_config_backup_info_data(&self, id: u32) -> Option<dto::RepoConfigBackupInfo> {
        self.get_data(id, self.subscription_data.repo_config_backup_info.clone())
            .flatten()
    }

    pub fn repo_config_backup_generate(&self, backup_id: u32, password: String) {
        let _ = self.vault.repo_config_backup_generate(backup_id, &password);
    }

    pub fn repo_config_backup_destroy(&self, backup_id: u32) {
        self.vault.repo_config_backup_destroy(backup_id);
    }

    // repo_space_usage

    pub fn repo_space_usage_create(&self, repo_id: String) -> u32 {
        self.vault.repo_space_usage_create(RepoId(repo_id))
    }

    pub fn repo_space_usage_info_subscribe(&self, usage_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoSpaceUsage],
            cb,
            self.subscription_data.repo_space_usage_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_space_usage::selectors::select_info(state, usage_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_space_usage_info_data(&self, id: u32) -> Option<dto::RepoSpaceUsageInfo> {
        self.get_data(id, self.subscription_data.repo_space_usage_info.clone())
            .flatten()
    }

    pub fn repo_space_usage_calculate(&self, usage_id: u32) {
        self.spawn(move |vault| {
            async move {
                let _ = vault.repo_space_usage_calculate(usage_id).await;
            }
            .boxed()
        });
    }

    pub fn repo_space_usage_destroy(&self, usage_id: u32) {
        self.vault.repo_space_usage_destroy(usage_id);
    }

    // repo_files

    pub fn repo_files_file_subscribe(&self, file_id: String, cb: Callback) -> u32 {
        let file_id = RepoFileId(file_id);

        self.subscribe(
            &[Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_file.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_files::selectors::select_file(state, &file_id).map(Into::into)
                })
            },
        )
    }

    pub fn repo_files_file_data(&self, id: u32) -> Option<dto::RepoFile> {
        self.get_data(id, self.subscription_data.repo_files_file.clone())
            .flatten()
    }

    pub fn repo_files_delete_file(&self, repo_id: String, encrypted_path: String) {
        self.spawn_result(move |vault| {
            async move {
                match vault
                    .repo_files_delete_files(&[(RepoId(repo_id), EncryptedPath(encrypted_path))])
                    .await
                {
                    Err(repo_files::errors::DeleteFileError::Canceled) => Ok(()),
                    res => res,
                }
            }
            .boxed()
        });
    }

    pub fn repo_files_rename_file(&self, repo_id: String, encrypted_path: String) {
        self.spawn_result(move |vault| {
            async move {
                vault
                    .repo_files_rename_file(&RepoId(repo_id), &EncryptedPath(encrypted_path))
                    .await
            }
            .boxed()
        });
    }

    pub fn repo_files_encrypt_name(&self, repo_id: String, name: String) -> Option<String> {
        self.vault
            .repo_files_service
            .encrypt_filename(&RepoId(repo_id), &DecryptedName(name))
            .map(|x| x.0)
            .ok()
    }

    // transfers

    pub fn transfers_is_active_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_is_active.clone(),
            move |vault| vault.with_state(|state| transfers::selectors::select_is_active(state)),
        )
    }

    pub fn transfers_is_active_data(&self, id: u32) -> bool {
        self.get_data(id, self.subscription_data.transfers_is_active.clone())
            .unwrap_or(false)
    }

    pub fn transfers_summary_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_summary.clone(),
            move |vault| {
                vault.with_state(|state| {
                    use transfers::selectors;

                    let now = vault.runtime.now();

                    dto::TransfersSummary {
                        total_count: state.transfers.total_count,
                        done_count: state.transfers.done_count,
                        failed_count: state.transfers.failed_count,
                        size_progress_display: files::file_size::size_of_display(
                            state.transfers.done_bytes,
                            state.transfers.total_bytes,
                        ),
                        percentage: selectors::select_percentage(state),
                        remaining_time_display: selectors::select_remaining_time(state, now)
                            .to_string(),
                        speed_display: files::file_size::speed_display_bytes_duration(
                            selectors::select_bytes_done(state),
                            selectors::select_duration(state, now),
                        ),
                        is_transferring: selectors::select_is_transferring(state),
                        is_all_done: selectors::select_is_all_done(state),
                        can_retry_all: selectors::select_can_retry_all(state),
                        can_abort_all: selectors::select_can_abort_all(state),
                    }
                })
            },
        )
    }

    pub fn transfers_summary_data(&self, id: u32) -> Option<dto::TransfersSummary> {
        self.get_data(id, self.subscription_data.transfers_summary.clone())
    }

    pub fn transfers_list_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_list.clone(),
            move |vault| {
                let now = vault.runtime.now();

                vault.with_state(|state| dto::TransfersList {
                    transfers: transfers::selectors::select_transfers(state)
                        .into_iter()
                        .map(|transfer| (transfer, now).into())
                        .collect(),
                })
            },
        )
    }

    pub fn transfers_list_data(&self, id: u32) -> Option<dto::TransfersList> {
        self.get_data(id, self.subscription_data.transfers_list.clone())
    }

    pub fn transfers_abort(&self, id: u32) {
        self.vault.transfers_abort(id);
    }

    pub fn transfers_abort_all(&self) {
        self.vault.transfers_abort_all();
    }

    pub fn transfers_retry(&self, id: u32) {
        self.vault.transfers_retry(id);
    }

    pub fn transfers_retry_all(&self) {
        self.vault.transfers_retry_all();
    }

    pub fn transfers_open(&self, id: u32) {
        self.spawn_result(move |vault| async move { vault.transfers_open(id).await }.boxed());
    }

    // dir_pickers

    pub fn dir_pickers_items_subscribe(&self, picker_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::DirPickers],
            cb,
            self.subscription_data.dir_pickers_items.clone(),
            move |vault| {
                vault.with_state(|state| {
                    state
                        .dir_pickers
                        .pickers
                        .get(&picker_id)
                        .map(|picker| picker.items.iter().map(From::from).collect())
                        .unwrap_or_else(|| Vec::new())
                })
            },
        )
    }

    pub fn dir_pickers_items_data(&self, id: u32) -> Option<Vec<dto::DirPickerItem>> {
        self.get_data(id, self.subscription_data.dir_pickers_items.clone())
    }

    // repo_files_browsers

    pub fn repo_files_browsers_create(
        &self,
        repo_id: String,
        encrypted_path: String,
        options: dto::RepoFilesBrowserOptions,
    ) -> u32 {
        let (browser_id, load_future) = self.vault.repo_files_browsers_create(
            RepoId(repo_id),
            &EncryptedPath(encrypted_path),
            options.into(),
        );

        let errors = self.errors.clone();

        self.vault
            .runtime
            .spawn(async move { errors.handle_result(load_future.await) }.boxed());

        browser_id
    }

    pub fn repo_files_browsers_destroy(&self, browser_id: u32) {
        self.vault.repo_files_browsers_destroy(browser_id);
    }

    pub fn repo_files_browsers_info(&self, browser_id: u32) -> Option<dto::RepoFilesBrowserInfo> {
        self.vault.with_state(|state| {
            repo_files_browsers::selectors::select_info(state, browser_id)
                .as_ref()
                .map(dto::RepoFilesBrowserInfo::from)
        })
    }

    pub fn repo_files_browsers_info_subscribe(&self, browser_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoFilesBrowsers, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_browsers_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_files_browsers::selectors::select_info(state, browser_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_files_browsers_info_data(&self, id: u32) -> Option<dto::RepoFilesBrowserInfo> {
        self.get_data(id, self.subscription_data.repo_files_browsers_info.clone())
            .flatten()
    }

    pub fn repo_files_browsers_load_files(&self, browser_id: u32) {
        self.spawn_result(move |vault| {
            async move { vault.repo_files_browsers_load_files(browser_id).await }.boxed()
        });
    }

    pub fn repo_files_browsers_select_file(
        &self,
        browser_id: u32,
        file_id: String,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.vault.repo_files_browsers_select_file(
            browser_id,
            RepoFileId(file_id),
            extend,
            range,
            force,
        );
    }

    pub fn repo_files_browsers_select_all(&self, browser_id: u32) {
        self.vault.repo_files_browsers_select_all(browser_id);
    }

    pub fn repo_files_browsers_clear_selection(&self, browser_id: u32) {
        self.vault.repo_files_browsers_clear_selection(browser_id);
    }

    pub fn repo_files_browsers_sort_by(&self, browser_id: u32, field: dto::RepoFilesSortField) {
        self.vault
            .repo_files_browsers_sort_by(browser_id, field.into(), None);
    }

    pub fn repo_files_browsers_create_dir(&self, browser_id: u32) {
        self.spawn_result(move |vault| {
            async move {
                match vault.repo_files_browsers_create_dir(browser_id).await {
                    Ok(_) => Ok(()),
                    Err(repo_files::errors::CreateDirError::Canceled) => Ok(()),
                    Err(err) => Err(err),
                }
            }
            .boxed()
        });
    }

    pub async fn repo_files_browsers_create_file(
        &self,
        browser_id: u32,
        name: String,
    ) -> Option<String> {
        match self
            .vault
            .repo_files_browsers_create_file(browser_id, &name)
            .await
        {
            Ok((_, path)) => Some(path.0),
            Err(repo_files::errors::CreateFileError::Canceled) => None,
            Err(err) => {
                self.handle_error(err);

                None
            }
        }
    }

    pub fn repo_files_browsers_delete_selected(&self, browser_id: u32) {
        self.spawn_result(move |vault| {
            async move {
                match vault.repo_files_browsers_delete_selected(browser_id).await {
                    Err(repo_files::errors::DeleteFileError::Canceled) => Ok(()),
                    res => res,
                }
            }
            .boxed()
        });
    }

    pub fn repo_files_browsers_move_selected(&self, browser_id: u32, mode: dto::RepoFilesMoveMode) {
        self.spawn_result(move |vault| {
            async move {
                vault
                    .repo_files_browsers_move_selected(browser_id, mode.into())
                    .await
            }
            .boxed()
        });
    }

    // repo_files_details

    pub fn repo_files_details_create(
        &self,
        repo_id: String,
        encrypted_path: String,
        is_editing: bool,
        options: dto::RepoFilesDetailsOptions,
    ) -> u32 {
        let (details_id, load_future) = self.vault.repo_files_details_create(
            RepoId(repo_id),
            &EncryptedPath(encrypted_path),
            is_editing,
            options.into(),
        );

        self.vault.runtime.spawn(
            async move {
                // error is displayed in the details component
                let _ = load_future.await;
            }
            .boxed(),
        );

        details_id
    }

    pub fn repo_files_details_destroy(&self, details_id: u32) {
        self.spawn_result(move |vault| {
            async move { vault.clone().repo_files_details_destroy(details_id).await }.boxed()
        });
    }

    pub fn repo_files_details_info_subscribe(&self, details_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoFilesDetails, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_details_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_files_details::selectors::select_info(state, details_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_files_details_info_data(&self, id: u32) -> Option<dto::RepoFilesDetailsInfo> {
        self.get_data(id, self.subscription_data.repo_files_details_info.clone())
            .flatten()
    }

    pub fn repo_files_details_file_subscribe(&self, details_id: u32, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoFilesDetails, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_details_file.clone(),
            move |vault| {
                vault.with_state(|state| {
                    repo_files_details::selectors::select_file(state, details_id).map(Into::into)
                })
            },
        )
    }

    pub fn repo_files_details_file_data(&self, id: u32) -> Option<dto::RepoFile> {
        self.get_data(id, self.subscription_data.repo_files_details_file.clone())
            .flatten()
    }

    pub fn repo_files_details_content_bytes_subscribe(&self, details_id: u32, cb: Callback) -> u32 {
        self.subscribe_changed(
            &[Event::RepoFilesDetailsContentData],
            cb,
            self.subscription_data
                .repo_files_details_content_bytes
                .clone(),
            move |vault, entry| {
                vault.with_state(|state| {
                    let (bytes, version) =
                        repo_files_details::selectors::select_content_bytes_version(
                            state, details_id,
                        );

                    store::update_if(
                        entry,
                        || dto::Versioned {
                            value: bytes.map(ToOwned::to_owned),
                            version,
                        },
                        |x| x.version != version,
                    )
                })
            },
        )
    }

    pub fn repo_files_details_content_bytes_data(&self, id: u32) -> Option<Vec<u8>> {
        self.get_data(
            id,
            self.subscription_data
                .repo_files_details_content_bytes
                .clone(),
        )
        .map(|data| data.value)
        .flatten()
    }

    pub fn repo_files_details_load_file(&self, details_id: u32) {
        self.spawn(move |vault| {
            async move {
                // error is displayed in the details component
                let _ = vault.clone().repo_files_details_load_file(details_id).await;
            }
            .boxed()
        });
    }

    pub fn repo_files_details_load_content(&self, details_id: u32) {
        self.spawn(move |vault| {
            async move {
                // error is displayed in the details component
                let _ = vault
                    .clone()
                    .repo_files_details_load_content(details_id)
                    .await;
            }
            .boxed()
        });
    }

    pub fn repo_files_details_edit(&self, details_id: u32) {
        self.vault.repo_files_details_edit(details_id);
    }

    pub fn repo_files_details_edit_cancel(&self, details_id: u32) {
        self.spawn(move |vault| {
            async move {
                // error is displayed in the details component
                let _ = vault
                    .clone()
                    .repo_files_details_edit_cancel(details_id)
                    .await;
            }
            .boxed()
        });
    }

    pub fn repo_files_details_set_content(&self, details_id: u32, content: Vec<u8>) {
        self.handle_result(
            self.vault
                .repo_files_details_set_content(details_id, content),
        );
    }

    pub fn repo_files_details_save(&self, details_id: u32) {
        self.spawn(move |vault| {
            async move {
                // error is displayed in the details component
                let _ = vault.clone().repo_files_details_save(details_id).await;
            }
            .boxed()
        });
    }

    pub fn repo_files_details_delete(&self, details_id: u32) {
        self.spawn_result(move |vault| {
            async move {
                match vault.repo_files_details_delete(details_id).await {
                    Err(repo_files::errors::DeleteFileError::Canceled) => Ok(()),
                    res => res,
                }
            }
            .boxed()
        });
    }

    // repo_files_move

    pub fn repo_files_move_info_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::RepoFilesMove, Event::RepoFiles, Event::DirPickers],
            cb,
            self.subscription_data.repo_files_move_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    state
                        .repo_files_move
                        .as_ref()
                        .map(|files_move| dto::RepoFilesMoveInfo {
                            src_files_count: files_move.src_paths.len(),
                            mode: (&files_move.mode).into(),
                            dir_picker_id: files_move.dir_picker_id,
                            dest_file_name: repo_files_move::selectors::select_dest_file(state)
                                .and_then(|file| {
                                    repo_files::selectors::select_file_name(state, file).ok()
                                })
                                .map(|x| x.0.to_owned()),
                            create_dir_enabled:
                                repo_files_move::selectors::select_create_dir_enabled(state),
                            can_move: repo_files_move::selectors::select_check_move(state).is_ok(),
                        })
                })
            },
        )
    }

    pub fn repo_files_move_info_data(&self, id: u32) -> Option<dto::RepoFilesMoveInfo> {
        self.get_data(id, self.subscription_data.repo_files_move_info.clone())
            .flatten()
    }

    pub fn repo_files_move_dir_picker_click(&self, item_id: String, is_arrow: bool) {
        self.spawn_result(move |vault| {
            async move {
                vault
                    .repo_files_move_dir_picker_click(&DirPickerItemId(item_id), is_arrow)
                    .await
            }
            .boxed()
        });
    }

    pub fn repo_files_move_move_files(&self) {
        self.spawn_result(move |vault| {
            async move { vault.repo_files_move_move_files().await }.boxed()
        });
    }

    pub fn repo_files_move_cancel(&self) {
        self.vault.repo_files_move_cancel();
    }

    pub fn repo_files_move_create_dir(&self) {
        self.spawn_result(move |vault| {
            async move {
                match vault.repo_files_move_create_dir().await {
                    Err(repo_files::errors::CreateDirError::Canceled) => Ok(()),
                    res => res,
                }
            }
            .boxed()
        });
    }

    // space_usage

    pub fn space_usage_subscribe(&self, cb: Callback) -> u32 {
        self.subscribe(
            &[Event::SpaceUsage],
            cb,
            self.subscription_data.space_usage.clone(),
            move |vault| {
                vault.with_state(|state| state.space_usage.space_usage.as_ref().map(Into::into))
            },
        )
    }

    pub fn space_usage_data(&self, id: u32) -> Option<dto::SpaceUsage> {
        self.get_data(id, self.subscription_data.space_usage.clone())
            .flatten()
    }
}
