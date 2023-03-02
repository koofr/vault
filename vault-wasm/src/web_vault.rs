use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use vault_core::store::Event;

use crate::browser_eventstream_websocket_client::{
    BrowserEventstreamWebSocketClient, BrowserEventstreamWebSocketDelegate,
};
use crate::browser_http_client::{BrowserHttpClient, BrowserHttpClientDelegate};
use crate::browser_runtime::BrowserRuntime;
use crate::browser_secure_storage::BrowserSecureStorage;
use crate::dto;
use crate::helpers;
use crate::uploadable::Uploadable;

#[wasm_bindgen(typescript_custom_section)]
const FILE_STREAM: &'static str = r#"
export interface FileStream {
  name: string;
  stream?: ReadableStream;
  blob?: Blob;
  size?: bigint;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Status")]
    pub type Status;

    #[wasm_bindgen(typescript_type = "Notification[]")]
    pub type NotificationVec;

    #[wasm_bindgen(typescript_type = "File | Blob")]
    pub type FileOrBlob;

    #[wasm_bindgen(typescript_type = "User | undefined")]
    pub type UserOption;

    #[wasm_bindgen(typescript_type = "Uint8Array | undefined")]
    pub type UserProfilePicture;

    #[wasm_bindgen(typescript_type = "Repo | undefined")]
    pub type RepoOption;

    #[wasm_bindgen(typescript_type = "Repos")]
    pub type Repos;

    #[wasm_bindgen(typescript_type = "RepoCreateInfo | undefined")]
    pub type RepoCreateInfoOption;

    #[wasm_bindgen(typescript_type = "RepoUnlockInfo | undefined")]
    pub type RepoUnlockInfoOption;

    #[wasm_bindgen(typescript_type = "RepoRemoveInfo | undefined")]
    pub type RepoRemoveInfoOption;

    #[wasm_bindgen(typescript_type = "RepoConfigBackupInfo | undefined")]
    pub type RepoConfigBackupInfoOption;

    #[wasm_bindgen(typescript_type = "RepoFile | undefined")]
    pub type RepoFileOption;

    #[wasm_bindgen(typescript_type = "RepoFilesBreadcrumb[]")]
    pub type RepoFilesBreadcrumbVec;

    #[wasm_bindgen(typescript_type = "RepoFilesUploadResult | Option")]
    pub type RepoFilesUploadResultOption;

    #[wasm_bindgen(typescript_type = "RepoFilesBrowserInfo | undefined")]
    pub type RepoFilesBrowserInfoOption;

    #[wasm_bindgen(typescript_type = "RepoFilesBrowserItem[]")]
    pub type RepoFilesBrowserItemVec;

    #[wasm_bindgen(typescript_type = "RepoFilesMoveInfo | undefined")]
    pub type RepoFilesMoveInfoOption;

    #[wasm_bindgen(typescript_type = "UploadsSummary")]
    pub type UploadsSummary;

    #[wasm_bindgen(typescript_type = "UploadsFiles")]
    pub type UploadsFiles;

    #[wasm_bindgen(typescript_type = "FileStream | undefined")]
    pub type FileStreamOption;

    #[wasm_bindgen(typescript_type = "DirPickerItem[]")]
    pub type DirPickerItemVec;

    #[wasm_bindgen(typescript_type = "SpaceUsage | undefined")]
    pub type SpaceUsageOption;
}

pub fn to_js<In: serde::ser::Serialize + ?Sized, Out: From<JsValue> + Into<JsValue>>(
    value: &In,
) -> Out {
    serde_wasm_bindgen::to_value(value).unwrap().into()
}

type Data<T> = Arc<Mutex<HashMap<u32, T>>>;

#[derive(Default)]
struct SubscriptionData {
    notifications: Data<Vec<dto::Notification>>,
    oauth2_status: Data<dto::Status>,
    user: Data<Option<dto::User>>,
    user_profile_picture_loaded: Data<bool>,
    repos: Data<dto::Repos>,
    repos_repo: Data<Option<dto::Repo>>,
    repo_create_info: Data<Option<dto::RepoCreateInfo>>,
    repo_unlock_info: Data<Option<dto::RepoUnlockInfo>>,
    repo_remove_info: Data<Option<dto::RepoRemoveInfo>>,
    repo_config_backup_info: Data<Option<dto::RepoConfigBackupInfo>>,
    repo_files_file: Data<Option<dto::RepoFile>>,
    uploads_is_active: Data<bool>,
    uploads_summary: Data<dto::UploadsSummary>,
    uploads_files: Data<dto::UploadsFiles>,
    dir_pickers_items: Data<Vec<dto::DirPickerItem>>,
    repo_files_browsers_info: Data<Option<dto::RepoFilesBrowserInfo>>,
    repo_files_browsers_items: Data<Vec<dto::RepoFilesBrowserItem>>,
    repo_files_browsers_breadcrumbs: Data<Vec<dto::RepoFilesBreadcrumb>>,
    repo_files_move_info: Data<Option<dto::RepoFilesMoveInfo>>,
    space_usage: Data<Option<dto::SpaceUsage>>,
}

#[wasm_bindgen]
pub struct WebVault {
    vault: Arc<vault_core::Vault>,
    errors: Arc<WebVaultErrors>,
    subscription_data: SubscriptionData,
    subscription_data_cleanups: Arc<Mutex<HashMap<u32, Box<dyn Fn() + Send + Sync + 'static>>>>,
    window: web_sys::Window,
}

#[wasm_bindgen]
impl WebVault {
    #[wasm_bindgen(constructor)]
    pub fn new(
        base_url: String,
        oauth2_client_id: String,
        oauth2_client_secret: String,
        oauth2_redirect_uri: String,
        browser_http_client_delegate: BrowserHttpClientDelegate,
        browser_eventstream_websocket_delegate: BrowserEventstreamWebSocketDelegate,
    ) -> Self {
        let oauth2_config = vault_core::oauth2::OAuth2Config {
            base_url: base_url.clone(),
            client_id: oauth2_client_id,
            client_secret: oauth2_client_secret,
            redirect_uri: oauth2_redirect_uri,
        };

        let vault = Arc::new(vault_core::Vault::new(
            base_url,
            oauth2_config,
            Box::new(BrowserHttpClient::new(browser_http_client_delegate)),
            Box::new(BrowserEventstreamWebSocketClient::new(
                browser_eventstream_websocket_delegate,
            )),
            Box::new(BrowserSecureStorage::new()),
            Box::new(BrowserRuntime::new()),
        ));

        let errors = Arc::new(WebVaultErrors::new(vault.clone()));

        Self {
            vault,
            errors,
            subscription_data: SubscriptionData::default(),
            subscription_data_cleanups: Arc::new(Mutex::new(HashMap::new())),
            window: web_sys::window().unwrap(),
        }
    }

    // errors

    fn handle_error(&self, user_error: impl vault_core::user_error::UserError) {
        self.errors.handle_error(user_error)
    }

    fn handle_result(&self, result: Result<(), impl vault_core::user_error::UserError>) {
        self.errors.handle_result(result)
    }

    // subscription

    fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        js_callback: js_sys::Function,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>) -> T + 'static,
    ) -> u32 {
        let window = self.window.clone();

        let id = self.vault.get_next_id();

        let generate_data = Arc::new(generate_data);

        let callback_vault = self.vault.clone();
        let callback_subscription_data = subscription_data.clone();
        let callback_generate_data = generate_data.clone();

        let store_callback: Box<dyn Fn() + 'static> = Box::new(move || {
            let new_value = callback_generate_data(callback_vault.clone());

            let callback_subscription_data = callback_subscription_data.clone();
            let mut subscription_data = callback_subscription_data.lock().unwrap();
            let current_data = subscription_data.get(&id);

            if current_data.is_none() || new_value != *current_data.unwrap() {
                subscription_data.insert(id, new_value.clone());

                // unlock subscription_data before calling the callback
                drop(subscription_data);

                window.set_timeout_with_callback(&js_callback).unwrap();
            }
        });
        let store_callback: Box<dyn Fn() + Send + Sync + 'static> = unsafe {
            Box::from_raw(Box::into_raw(store_callback) as *mut (dyn Fn() + Send + Sync + 'static))
        };

        self.vault.on(id, events, store_callback);

        subscription_data
            .lock()
            .unwrap()
            .insert(id, generate_data(self.vault.clone()));

        let cleanup_subscription_data = subscription_data.clone();

        let cleanup = Box::new(move || {
            cleanup_subscription_data
                .clone()
                .lock()
                .unwrap()
                .remove(&id);
        });

        self.subscription_data_cleanups
            .lock()
            .unwrap()
            .insert(id, cleanup);

        id
    }

    fn get_data<T: Clone + PartialEq + Send>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Option<T> {
        subscription_data.lock().unwrap().get(&id).cloned()
    }

    fn get_data_js<T: Clone + PartialEq + Send + Serialize, Out: From<JsValue> + Into<JsValue>>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Out {
        to_js(&self.get_data(id, subscription_data))
    }

    #[wasm_bindgen(js_name = unsubscribe)]
    pub fn unsubscribe(&self, id: u32) {
        self.vault.remove_listener(id);

        let cleanup = self.subscription_data_cleanups.lock().unwrap().remove(&id);

        if let Some(cleanup) = cleanup {
            cleanup();
        }
    }

    // lifecycle

    #[wasm_bindgen(js_name = load)]
    pub async fn load(&self) {
        self.handle_result(self.vault.load().await)
    }

    #[wasm_bindgen(js_name = logout)]
    pub fn logout(&self) {
        self.vault.logout()
    }

    // notifications

    #[wasm_bindgen(js_name = notificationsSubscribe)]
    pub fn notifications_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Notifications],
            cb,
            self.subscription_data.notifications.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::notifications::selectors::select_notifications(state)
                        .into_iter()
                        .map(Into::into)
                        .collect()
                })
            },
        )
    }

    #[wasm_bindgen(js_name = notificationsData)]
    pub fn notifications_data(&self, id: u32) -> NotificationVec {
        self.get_data_js(id, self.subscription_data.notifications.clone())
    }

    #[wasm_bindgen(js_name = notificationsRemove)]
    pub fn notifications_remove(&self, id: u32) {
        self.vault.notifications_remove(id)
    }

    #[wasm_bindgen(js_name = notificationsRemoveAll)]
    pub fn notifications_remove_all(&self) {
        self.vault.notifications_remove_all()
    }

    // oauth2

    #[wasm_bindgen(js_name = oauth2StatusSubscribe)]
    pub fn oauth2_status_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Auth],
            cb,
            self.subscription_data.oauth2_status.clone(),
            move |vault| {
                vault.with_state(|state| vault_core::oauth2::selectors::select_status(state).into())
            },
        )
    }

    #[wasm_bindgen(js_name = oauth2StatusData)]
    pub fn oauth2_status_data(&self, id: u32) -> Status {
        self.get_data_js(id, self.subscription_data.oauth2_status.clone())
    }

    #[wasm_bindgen(js_name = oauth2StartFlow)]
    pub fn oauth2_start_flow(&self) -> String {
        self.vault.oauth2_start_flow()
    }

    #[wasm_bindgen(js_name = oauth2FinishFlowUrl)]
    pub async fn oauth2_finish_flow_url(&self, url: &str) -> bool {
        let res = self.vault.oauth2_finish_flow_url(url).await;

        let success = res.is_ok();

        self.handle_result(res);

        success
    }

    // config

    #[wasm_bindgen(js_name = configGetBaseUrl)]
    pub fn config_get_base_url(&self) -> String {
        self.vault.with_state(|state| state.config.base_url.clone())
    }

    // user

    #[wasm_bindgen(js_name = userSubscribe)]
    pub fn user_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::User],
            cb,
            self.subscription_data.user.clone(),
            move |vault| vault.with_state(|state| state.user.user.as_ref().map(Into::into)),
        )
    }

    #[wasm_bindgen(js_name = userData)]
    pub fn user_data(&self, id: u32) -> UserOption {
        self.get_data_js(id, self.subscription_data.user.clone())
    }

    #[wasm_bindgen(js_name = userProfilePictureLoadedSubscribe)]
    pub fn user_profile_picture_loaded_subscribe(&self, cb: js_sys::Function) -> u32 {
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
                            vault_core::common::state::Status::Loaded => true,
                            _ => false,
                        })
                        .unwrap_or(false)
                })
            },
        )
    }

    #[wasm_bindgen(js_name = userProfilePictureLoadedData)]
    pub fn user_profile_picture_loaded_data(&self, id: u32) -> bool {
        self.get_data(
            id,
            self.subscription_data.user_profile_picture_loaded.clone(),
        )
        .unwrap_or(false)
    }

    #[wasm_bindgen(js_name = userGetProfilePicture)]
    pub fn user_get_profile_picture(&self) -> UserProfilePicture {
        UserProfilePicture::from(self.vault.with_state(|state| {
            match state
                .user
                .user
                .as_ref()
                .and_then(|user| user.profile_picture_bytes.as_ref())
            {
                Some(bytes) => helpers::bytes_to_array(&bytes),
                None => JsValue::UNDEFINED,
            }
        }))
    }

    #[wasm_bindgen(js_name = userEnsureProfilePicture)]
    pub async fn user_ensure_profile_picture(&self) {
        self.handle_result(self.vault.user_ensure_profile_picture().await)
    }

    // repos

    #[wasm_bindgen(js_name = reposSubscribe)]
    pub fn repos_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Repos],
            cb,
            self.subscription_data.repos.clone(),
            move |vault| vault.with_state(|state| dto::Repos::from(state)),
        )
    }

    #[wasm_bindgen(js_name = reposData)]
    pub fn repos_data(&self, id: u32) -> Repos {
        self.get_data_js(id, self.subscription_data.repos.clone())
    }

    #[wasm_bindgen(js_name = reposRepoSubscribe)]
    pub fn repos_repo_subscribe(&self, repo_id: String, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Repos],
            cb,
            self.subscription_data.repos_repo.clone(),
            move |vault| {
                vault.with_state(|state| state.repos.repos_by_id.get(&repo_id).map(dto::Repo::from))
            },
        )
    }

    #[wasm_bindgen(js_name = reposRepoData)]
    pub fn repos_repo_data(&self, id: u32) -> RepoOption {
        self.get_data_js(id, self.subscription_data.repos_repo.clone())
    }

    #[wasm_bindgen(js_name = reposLockRepo)]
    pub fn repos_lock_repo(&self, repo_id: &str) {
        self.handle_result(self.vault.repos_lock_repo(repo_id))
    }

    // repo_create

    #[wasm_bindgen(js_name = repoCreateInfoSubscribe)]
    pub fn repo_create_info_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::RepoCreate, Event::DirPickers],
            cb,
            self.subscription_data.repo_create_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    state
                        .repo_create
                        .as_ref()
                        .map(|repo_create| match repo_create {
                            vault_core::repo_create::state::RepoCreateState::Form(form) => {
                                let location_breadcrumbs = form
                                    .location
                                    .as_ref()
                                    .map(|location| {
                                        vault_core::remote_files::selectors::select_breadcrumbs(
                                            state,
                                            &location.mount_id,
                                            &location.path,
                                        )
                                    })
                                    .unwrap_or(Vec::new());

                                dto::RepoCreateInfo {
                                    form: Some(dto::RepoCreateForm {
                                        init_status: (&form.init_status).into(),
                                        location: form
                                            .location
                                            .as_ref()
                                            .map(|location| location.into()),
                                        location_breadcrumbs: location_breadcrumbs
                                            .iter()
                                            .map(dto::RemoteFilesBreadcrumb::from)
                                            .collect(),
                                        location_dir_picker_id: form.location_dir_picker_id,
                                        location_dir_picker_can_select: vault_core::repo_create::selectors::select_location_dir_picker_can_select(
                                            state,
                                        ),
                                        location_dir_picker_can_show_create_dir: vault_core::repo_create::selectors::select_location_dir_picker_can_show_create_dir(
                                            state,
                                        ),
                                        password: form.password.clone(),
                                        salt: form.salt.clone(),
                                        fill_from_rclone_config_error: form
                                            .fill_from_rclone_config_error
                                            .as_ref()
                                            .map(|e| e.to_string()),
                                        can_create:
                                            vault_core::repo_create::selectors::select_can_create(
                                                state,
                                            ),
                                        create_status: (&form.create_status).into(),
                                    }),
                                    created: None,
                                }
                            }
                            vault_core::repo_create::state::RepoCreateState::Created(created) => {
                                dto::RepoCreateInfo {
                                    form: None,
                                    created: Some(created.into()),
                                }
                            }
                        })
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoCreateInfoData)]
    pub fn repo_create_info_data(&self, id: u32) -> RepoCreateInfoOption {
        self.get_data_js(id, self.subscription_data.repo_create_info.clone())
    }

    #[wasm_bindgen(js_name = repoCreateInit)]
    pub async fn repo_create_init(&self) {
        self.vault.repo_create_init().await;
    }

    #[wasm_bindgen(js_name = repoCreateReset)]
    pub fn repo_create_reset(&self) {
        self.vault.repo_create_reset();
    }

    #[wasm_bindgen(js_name = repoCreateSetLocation)]
    pub fn repo_create_set_location(&self, mount_id: String, path: String) {
        self.vault
            .repo_create_set_location(vault_core::remote_files::state::RemoteFilesLocation {
                mount_id,
                path,
            })
    }

    #[wasm_bindgen(js_name = repoCreateSetPassword)]
    pub fn repo_create_set_password(&self, password: String) {
        self.vault.repo_create_set_password(password)
    }

    #[wasm_bindgen(js_name = repoCreateSetSalt)]
    pub fn repo_create_set_salt(&self, salt: Option<String>) {
        self.vault.repo_create_set_salt(salt)
    }

    #[wasm_bindgen(js_name = repoCreateFillFromRcloneConfig)]
    pub fn repo_create_fill_from_rclone_config(&self, config: String) {
        self.vault.repo_create_fill_from_rclone_config(config)
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerShow)]
    pub async fn repo_create_location_dir_picker_show(&self) {
        self.handle_result(self.vault.repo_create_location_dir_picker_show().await)
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerSelect)]
    pub fn repo_create_location_dir_picker_select(&self) {
        self.vault.repo_create_location_dir_picker_select()
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerCancel)]
    pub fn repo_create_location_dir_picker_cancel(&self) {
        self.vault.repo_create_location_dir_picker_cancel()
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerCanCreateDir)]
    pub fn repo_create_location_dir_picker_can_create_dir(&self, name: &str) -> bool {
        self.vault
            .repo_create_location_dir_picker_check_create_dir(name)
            .is_ok()
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerCreateDir)]
    pub async fn repo_create_location_dir_picker_create_dir(&self, name: &str) {
        self.handle_result(
            self.vault
                .repo_create_location_dir_picker_create_dir(name)
                .await,
        )
    }

    #[wasm_bindgen(js_name = repoCreateCreate)]
    pub async fn repo_create_create(&self) {
        self.vault.repo_create_create().await
    }

    // repo_unlock

    #[wasm_bindgen(js_name = repoUnlockInfoSubscribe)]
    pub fn repo_unlock_info_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::RepoUnlock],
            cb,
            self.subscription_data.repo_unlock_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_unlock::selectors::select_info(state).map(|info| {
                        dto::RepoUnlockInfo {
                            status: info.status.into(),
                            repo_name: info.repo_name.map(str::to_string),
                        }
                    })
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoUnlockInfoData)]
    pub fn repo_unlock_info_data(&self, id: u32) -> RepoUnlockInfoOption {
        self.get_data_js(id, self.subscription_data.repo_unlock_info.clone())
    }

    #[wasm_bindgen(js_name = repoUnlockInit)]
    pub fn repo_unlock_init(&self, repo_id: &str) {
        self.vault.repo_unlock_init(repo_id)
    }

    #[wasm_bindgen(js_name = repoUnlockUnlock)]
    pub async fn repo_unlock_unlock(&self, password: &str) {
        let _ = self.vault.repo_unlock_unlock(password).await;
    }

    #[wasm_bindgen(js_name = repoUnlockDestroy)]
    pub fn repo_unlock_destroy(&self, repo_id: &str) {
        self.vault.repo_unlock_destroy(repo_id)
    }

    // repo_remove

    #[wasm_bindgen(js_name = repoRemoveInfoSubscribe)]
    pub fn repo_remove_info_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::RepoRemove],
            cb,
            self.subscription_data.repo_remove_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_remove::selectors::select_info(state).map(|info| {
                        dto::RepoRemoveInfo {
                            status: info.status.into(),
                            repo_name: info.repo_name.map(str::to_string),
                        }
                    })
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoRemoveInfoData)]
    pub fn repo_remove_info_data(&self, id: u32) -> RepoRemoveInfoOption {
        self.get_data_js(id, self.subscription_data.repo_remove_info.clone())
    }

    #[wasm_bindgen(js_name = repoRemoveInit)]
    pub fn repo_remove_init(&self, repo_id: &str) {
        self.vault.repo_remove_init(repo_id)
    }

    #[wasm_bindgen(js_name = repoRemoveRemove)]
    pub async fn repo_remove_remove(&self, password: &str) -> bool {
        self.vault.repo_remove_remove(password).await.is_ok()
    }

    #[wasm_bindgen(js_name = repoRemoveDestroy)]
    pub fn repo_remove_destroy(&self, repo_id: &str) {
        self.vault.repo_remove_destroy(repo_id)
    }

    // repo_config_backup

    #[wasm_bindgen(js_name = repoConfigBackupInfoSubscribe)]
    pub fn repo_config_backup_info_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::RepoConfigBackup],
            cb,
            self.subscription_data.repo_config_backup_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_config_backup::selectors::select_info(state).map(|info| {
                        dto::RepoConfigBackupInfo {
                            status: info.status.into(),
                            config: info.config.map(Into::into),
                        }
                    })
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoConfigBackupInfoData)]
    pub fn repo_config_backup_info_data(&self, id: u32) -> RepoConfigBackupInfoOption {
        self.get_data_js(id, self.subscription_data.repo_config_backup_info.clone())
    }

    #[wasm_bindgen(js_name = repoConfigBackupInit)]
    pub fn repo_config_backup_init(&self, repo_id: &str) {
        self.vault.repo_config_backup_init(repo_id)
    }

    #[wasm_bindgen(js_name = repoConfigBackupGenerate)]
    pub async fn repo_config_backup_generate(&self, password: &str) {
        let _ = self.vault.repo_config_backup_generate(password).await;
    }

    #[wasm_bindgen(js_name = repoConfigBackupDestroy)]
    pub fn repo_config_backup_destroy(&self, repo_id: &str) {
        self.vault.repo_config_backup_destroy(repo_id)
    }

    // repo_files

    #[wasm_bindgen(js_name = repoFilesFileSubscribe)]
    pub fn repo_files_file_subscribe(&self, file_id: String, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_file.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files::selectors::select_file(state, &file_id).map(Into::into)
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoFilesFileData)]
    pub fn repo_files_file_data(&self, id: u32) -> RepoFileOption {
        self.get_data_js(id, self.subscription_data.repo_files_file.clone())
    }

    #[wasm_bindgen(js_name = repoFilesLoadFiles)]
    pub async fn repo_files_load_files(&self, repo_id: &str, path: &str) {
        self.handle_result(self.vault.repo_files_load_files(repo_id, path).await)
    }

    #[wasm_bindgen(js_name = repoFilesGetFileStream)]
    pub async fn repo_files_get_file_stream(
        &self,
        file_id: &str,
        force_blob: bool,
    ) -> FileStreamOption {
        let file_reader = match self.vault.repo_files_get_file_reader(file_id).await {
            Ok(file_reader) => file_reader,
            Err(err) => {
                self.handle_error(err);
                return JsValue::UNDEFINED.into();
            }
        };

        let file_stream = match helpers::reader_to_file_stream(
            &file_reader.name,
            file_reader.reader,
            Some(file_reader.size),
            force_blob,
        )
        .await
        {
            Ok(file_stream) => file_stream,
            Err(err) => {
                self.handle_error(err);
                return JsValue::UNDEFINED.into();
            }
        };

        FileStreamOption::from(file_stream)
    }

    #[wasm_bindgen(js_name = repoFilesDeleteFile)]
    pub async fn repo_files_delete_file(&self, repo_id: &str, path: &str) {
        self.handle_result(self.vault.repo_files_delete_file(repo_id, path).await)
    }

    #[wasm_bindgen(js_name = repoFilesCanRenameFile)]
    pub fn repo_files_can_rename_file(&self, repo_id: &str, path: &str, name: &str) -> bool {
        self.vault
            .repo_files_check_rename_file(repo_id, path, name)
            .is_ok()
    }

    #[wasm_bindgen(js_name = repoFilesRenameFile)]
    pub async fn repo_files_rename_file(&self, repo_id: &str, path: &str, name: &str) {
        self.handle_result(self.vault.repo_files_rename_file(repo_id, path, name).await)
    }

    // uploads

    #[wasm_bindgen(js_name = uploadsIsActiveSubscribe)]
    pub fn uploads_is_active_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Uploads],
            cb,
            self.subscription_data.uploads_is_active.clone(),
            move |vault| {
                vault.with_state(|state| vault_core::uploads::selectors::select_is_active(state))
            },
        )
    }

    #[wasm_bindgen(js_name = uploadsIsActiveData)]
    pub fn uploads_is_active_data(&self, id: u32) -> bool {
        self.get_data(id, self.subscription_data.uploads_is_active.clone())
            .unwrap_or(false)
    }

    #[wasm_bindgen(js_name = uploadsSummarySubscribe)]
    pub fn uploads_summary_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Uploads],
            cb,
            self.subscription_data.uploads_summary.clone(),
            move |vault| {
                vault.with_state(|state| {
                    use vault_core::uploads::selectors;

                    let now = instant::now() as i64;

                    dto::UploadsSummary {
                        total_count: state.uploads.total_count,
                        done_count: state.uploads.done_count,
                        failed_count: state.uploads.failed_count,
                        total_bytes: state.uploads.total_bytes,
                        done_bytes: state.uploads.done_bytes,
                        percentage: selectors::select_percentage(state),
                        remaining_time: (&selectors::select_remaining_time(state, now)).into(),
                        bytes_per_second: selectors::select_bytes_per_second(state, now),
                        is_uploading: selectors::select_is_uploading(state),
                        can_retry: selectors::select_can_retry(state),
                        can_abort: selectors::select_can_abort(state),
                    }
                })
            },
        )
    }

    #[wasm_bindgen(js_name = uploadsSummaryData)]
    pub fn uploads_summary_data(&self, id: u32) -> UploadsSummary {
        self.get_data_js(id, self.subscription_data.uploads_summary.clone())
    }

    #[wasm_bindgen(js_name = uploadsFilesSubscribe)]
    pub fn uploads_files_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::Uploads],
            cb,
            self.subscription_data.uploads_files.clone(),
            move |vault| {
                vault.with_state(|state| dto::UploadsFiles {
                    files: vault_core::uploads::selectors::select_files(state)
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                })
            },
        )
    }

    #[wasm_bindgen(js_name = uploadsFilesData)]
    pub fn uploads_files_data(&self, id: u32) -> UploadsFiles {
        self.get_data_js(id, self.subscription_data.uploads_files.clone())
    }

    #[wasm_bindgen(js_name = uploadsUpload)]
    pub async fn uploads_upload(
        &self,
        repo_id: &str,
        parent_path: &str,
        name: &str,
        file: FileOrBlob,
    ) -> RepoFilesUploadResultOption {
        let uploadable = Box::pin(Uploadable::from_value(file.into()).unwrap());

        match self
            .vault
            .uploads_upload(repo_id, parent_path, name, uploadable)
            .await
        {
            Ok(res) => to_js(&dto::RepoFilesUploadResult::from(res)),
            Err(_) => {
                // upload errors are displayed in uploads component
                JsValue::UNDEFINED.into()
            }
        }
    }

    #[wasm_bindgen(js_name = uploadsAbortFile)]
    pub fn uploads_abort_file(&self, id: u32) {
        self.vault.uploads_abort_file(id);
    }

    #[wasm_bindgen(js_name = uploadsAbortAll)]
    pub fn uploads_abort_all(&self) {
        self.vault.uploads_abort_all();
    }

    #[wasm_bindgen(js_name = uploadsRetryFile)]
    pub fn uploads_retry_file(&self, id: u32) {
        self.vault.uploads_retry_file(id);
    }

    #[wasm_bindgen(js_name = uploadsRetryAll)]
    pub fn uploads_retry_all(&self) {
        self.vault.uploads_retry_all();
    }

    // dir_pickers

    #[wasm_bindgen(js_name = dirPickersItemsSubscribe)]
    pub fn dir_pickers_items_subscribe(&self, picker_id: u32, cb: js_sys::Function) -> u32 {
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

    #[wasm_bindgen(js_name = dirPickersItemsData)]
    pub fn dir_pickers_items_data(&self, id: u32) -> DirPickerItemVec {
        self.get_data_js(id, self.subscription_data.dir_pickers_items.clone())
    }

    // remote_files_dir_pickers

    #[wasm_bindgen(js_name = remoteFilesDirPickersClick)]
    pub async fn remote_files_dir_pickers_click(
        &self,
        picker_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) {
        self.handle_result(
            self.vault
                .remote_files_dir_pickers_click(picker_id, item_id, is_arrow)
                .await,
        )
    }

    // repo_files_browsers

    #[wasm_bindgen(js_name = repoFilesBrowsersCreate)]
    pub fn repo_files_browsers_create(&self, repo_id: &str, path: &str) -> u32 {
        let (browser_id, load_future) = self.vault.repo_files_browsers_create(repo_id, path);

        let errors = self.errors.clone();

        spawn_local(async move { errors.handle_result(load_future.await) });

        browser_id
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersDestroy)]
    pub fn repo_files_browsers_destroy(&self, browser_id: u32) {
        self.vault.repo_files_browsers_destroy(browser_id)
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersInfo)]
    pub fn repo_files_browsers_info(&self, browser_id: u32) -> RepoFilesBrowserInfoOption {
        to_js(&self.vault.with_state(|state| {
            vault_core::repo_files_browsers::selectors::select_info(state, browser_id)
                .as_ref()
                .map(dto::RepoFilesBrowserInfo::from)
        }))
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersInfoSubscribe)]
    pub fn repo_files_browsers_info_subscribe(&self, browser_id: u32, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::RepoFilesBrowsers, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_browsers_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_browsers::selectors::select_info(state, browser_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersInfoData)]
    pub fn repo_files_browsers_info_data(&self, id: u32) -> RepoFilesBrowserInfoOption {
        self.get_data_js(id, self.subscription_data.repo_files_browsers_info.clone())
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersItemsSubscribe)]
    pub fn repo_files_browsers_items_subscribe(
        &self,
        browser_id: u32,
        cb: js_sys::Function,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoFilesBrowsers, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_browsers_items.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_browsers::selectors::select_items(state, browser_id)
                        .iter()
                        .map(|item| item.into())
                        .collect()
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersItemsData)]
    pub fn repo_files_browsers_items_data(&self, id: u32) -> RepoFilesBrowserItemVec {
        self.get_data_js(id, self.subscription_data.repo_files_browsers_items.clone())
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersBreadcrumbsSubscribe)]
    pub fn repo_files_browsers_breadcrumbs_subscribe(
        &self,
        browser_id: u32,
        cb: js_sys::Function,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoFilesBrowsers],
            cb,
            self.subscription_data
                .repo_files_browsers_breadcrumbs
                .clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_browsers::selectors::select_breadcrumbs(
                        state, browser_id,
                    )
                    .iter()
                    .map(Into::into)
                    .collect()
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersBreadcrumbsData)]
    pub fn repo_files_browsers_breadcrumbs_data(&self, id: u32) -> RepoFilesBreadcrumbVec {
        self.get_data_js(
            id,
            self.subscription_data
                .repo_files_browsers_breadcrumbs
                .clone(),
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersSetLocation)]
    pub async fn repo_files_browsers_set_location(
        &self,
        browser_id: u32,
        repo_id: &str,
        path: &str,
    ) {
        self.handle_result(
            self.vault
                .repo_files_browsers_set_location(browser_id, repo_id, path)
                .await,
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersLoadFiles)]
    pub async fn repo_files_browsers_load_files(&self, browser_id: u32) {
        self.handle_result(self.vault.repo_files_browsers_load_files(browser_id).await)
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersSelectFile)]
    pub fn repo_files_browsers_select_file(
        &self,
        browser_id: u32,
        file_id: &str,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.vault
            .repo_files_browsers_select_file(browser_id, file_id, extend, range, force)
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersToggleSelectAll)]
    pub fn repo_files_browsers_toggle_select_all(&self, browser_id: u32) {
        self.vault.repo_files_browsers_toggle_select_all(browser_id)
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersClearSelection)]
    pub fn repo_files_browsers_clear_selection(&self, browser_id: u32) {
        self.vault.repo_files_browsers_clear_selection(browser_id)
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersGetSelectedStream)]
    pub async fn repo_files_browsers_get_selected_stream(
        &self,
        browser_id: u32,
        force_blob: bool,
    ) -> FileStreamOption {
        let file_reader = match self
            .vault
            .repo_files_browsers_get_selected_stream(browser_id)
            .await
        {
            Ok(file_reader) => file_reader,
            Err(err) => {
                self.handle_error(err);
                return JsValue::UNDEFINED.into();
            }
        };

        let file_stream = match helpers::reader_to_file_stream(
            &file_reader.name,
            file_reader.reader,
            Some(file_reader.size),
            force_blob,
        )
        .await
        {
            Ok(file_stream) => file_stream,
            Err(err) => {
                self.handle_error(err);
                return JsValue::UNDEFINED.into();
            }
        };

        FileStreamOption::from(file_stream)
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersCanCreateDir)]
    pub fn repo_files_browsers_can_create_dir(&self, browser_id: u32, name: &str) -> bool {
        self.vault
            .repo_files_browsers_check_create_dir(browser_id, name)
            .is_ok()
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersCreateDir)]
    pub async fn repo_files_browsers_create_dir(&self, browser_id: u32, name: &str) {
        self.handle_result(
            self.vault
                .repo_files_browsers_create_dir(browser_id, name)
                .await,
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersDeleteSelected)]
    pub async fn repo_files_browsers_delete_selected(&self, browser_id: u32) {
        self.handle_result(
            self.vault
                .repo_files_browsers_delete_selected(browser_id)
                .await,
        )
    }

    // repo_files_move

    #[wasm_bindgen(js_name = repoFilesMoveInfoSubscribe)]
    pub fn repo_files_move_info_subscribe(&self, cb: js_sys::Function) -> u32 {
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
                            src_files_count: files_move.src_file_ids.len(),
                            mode: (&files_move.mode).into(),
                            dir_picker_id: files_move.dir_picker_id,
                            dest_file_name:
                                vault_core::repo_files_move::selectors::select_dest_file(state)
                                    .and_then(|file| {
                                        vault_core::repo_files::selectors::select_file_name(
                                            state, file,
                                        )
                                    })
                                    .map(str::to_string),
                            can_show_create_dir:
                                vault_core::repo_files_move::selectors::select_can_show_create_dir(
                                    state,
                                ),
                            can_move: vault_core::repo_files_move::selectors::select_check_move(
                                state,
                            )
                            .is_ok(),
                        })
                })
            },
        )
    }

    #[wasm_bindgen(js_name = repoFilesMoveInfoData)]
    pub fn repo_files_move_info_data(&self, id: u32) -> RepoFilesMoveInfoOption {
        self.get_data_js(id, self.subscription_data.repo_files_move_info.clone())
    }

    #[wasm_bindgen(js_name = repoFilesMoveShow)]
    pub async fn repo_files_move_show(&self, browser_id: u32, mode: dto::RepoFilesMoveMode) {
        self.handle_result(
            self.vault
                .repo_files_move_show(browser_id, mode.into())
                .await,
        )
    }

    #[wasm_bindgen(js_name = repoFilesMoveMoveFiles)]
    pub async fn repo_files_move_move_files(&self) {
        self.handle_result(self.vault.repo_files_move_move_files().await)
    }

    #[wasm_bindgen(js_name = repoFilesMoveCancel)]
    pub fn repo_files_move_cancel(&self) {
        self.vault.repo_files_move_cancel()
    }

    #[wasm_bindgen(js_name = repoFilesMoveCanCreateDir)]
    pub fn repo_files_move_can_create_dir(&self, name: &str) -> bool {
        self.vault.repo_files_move_check_create_dir(name).is_ok()
    }

    #[wasm_bindgen(js_name = repoFilesMoveCreateDir)]
    pub async fn repo_files_move_create_dir(&self, name: &str) {
        self.handle_result(self.vault.repo_files_move_create_dir(name).await)
    }

    // repo_files_dir_pickers

    #[wasm_bindgen(js_name = repoFilesDirPickersClick)]
    pub async fn repo_files_dir_pickers_click(
        &self,
        picker_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) {
        self.handle_result(
            self.vault
                .repo_files_dir_pickers_click(picker_id, item_id, is_arrow)
                .await,
        )
    }

    // space_usage

    #[wasm_bindgen(js_name = spaceUsageSubscribe)]
    pub fn space_usage_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.subscribe(
            &[Event::SpaceUsage],
            cb,
            self.subscription_data.space_usage.clone(),
            move |vault| {
                vault.with_state(|state| state.space_usage.space_usage.as_ref().map(Into::into))
            },
        )
    }

    #[wasm_bindgen(js_name = spaceUsageData)]
    pub fn space_usage_data(&self, id: u32) -> SpaceUsageOption {
        self.get_data_js(id, self.subscription_data.space_usage.clone())
    }
}

pub struct WebVaultErrors {
    vault: Arc<vault_core::Vault>,
}

impl WebVaultErrors {
    fn new(vault: Arc<vault_core::Vault>) -> Self {
        Self { vault }
    }

    pub fn handle_error_str(&self, error_str: String) {
        self.vault.notifications_show(error_str);
    }

    pub fn handle_error(&self, user_error: impl vault_core::user_error::UserError) {
        self.handle_error_str(user_error.user_error());
    }

    pub fn handle_result(&self, result: Result<(), impl vault_core::user_error::UserError>) {
        match result {
            Ok(()) => (),
            Err(err) => self.handle_error(err),
        }
    }
}
