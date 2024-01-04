use std::sync::Arc;

use wasm_bindgen::prelude::*;
use web_sys::{AbortSignal, Storage};

use vault_core::{
    transfers,
    types::{EncryptedPath, RepoId},
};
use vault_web_api::{dto, web_errors::WebErrors};

use crate::{
    browser_eventstream_websocket_client::{
        BrowserEventstreamWebSocketClient, BrowserEventstreamWebSocketDelegate,
    },
    browser_http_client::{BrowserHttpClient, BrowserHttpClientDelegate},
    browser_runtime::BrowserRuntime,
    browser_secure_storage::BrowserSecureStorage,
    browser_uploadable::BrowserUploadable,
    helpers,
};

#[wasm_bindgen(typescript_custom_section)]
const FILE_STREAM: &'static str = r#"
export interface FileStream {
  name: string;
  stream?: ReadableStream;
  blob?: Blob;
  size: SizeInfo;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "number[] | undefined")]
    pub type IdVecOption;

    #[wasm_bindgen(typescript_type = "Status | undefined")]
    pub type StatusOption;

    #[wasm_bindgen(typescript_type = "RelativeTime")]
    pub type RelativeTime;

    #[wasm_bindgen(typescript_type = "Notification[] | undefined")]
    pub type NotificationVecOption;

    #[wasm_bindgen(typescript_type = "Dialog | undefined")]
    pub type DialogOption;

    #[wasm_bindgen(typescript_type = "File | Blob")]
    pub type FileOrBlob;

    #[wasm_bindgen(typescript_type = "Uint8Array | undefined")]
    pub type Bytes;

    #[wasm_bindgen(typescript_type = "User | undefined")]
    pub type UserOption;

    #[wasm_bindgen(typescript_type = "FileIconProps")]
    pub type FileIconProps;

    #[wasm_bindgen(typescript_type = "RepoInfo | undefined")]
    pub type RepoInfoOption;

    #[wasm_bindgen(typescript_type = "Repos | undefined")]
    pub type ReposOption;

    #[wasm_bindgen(typescript_type = "RepoAutoLock")]
    pub type RepoAutoLock;

    #[wasm_bindgen(typescript_type = "RepoCreateInfo | undefined")]
    pub type RepoCreateInfoOption;

    #[wasm_bindgen(typescript_type = "RepoUnlockOptions")]
    pub type RepoUnlockOptions;

    #[wasm_bindgen(typescript_type = "RepoUnlockInfo | undefined")]
    pub type RepoUnlockInfoOption;

    #[wasm_bindgen(typescript_type = "RepoRemoveInfo | undefined")]
    pub type RepoRemoveInfoOption;

    #[wasm_bindgen(typescript_type = "RepoConfigBackupInfo | undefined")]
    pub type RepoConfigBackupInfoOption;

    #[wasm_bindgen(typescript_type = "RepoSpaceUsageInfo | undefined")]
    pub type RepoSpaceUsageInfoOption;

    #[wasm_bindgen(typescript_type = "RepoFile | undefined")]
    pub type RepoFileOption;

    #[wasm_bindgen(typescript_type = "RepoFilesBreadcrumb[]")]
    pub type RepoFilesBreadcrumbVec;

    #[wasm_bindgen(typescript_type = "RepoFilesUploadResult | undefined")]
    pub type RepoFilesUploadResultOption;

    #[wasm_bindgen(typescript_type = "RepoFilesBrowserOptions")]
    pub type RepoFilesBrowserOptions;

    #[wasm_bindgen(typescript_type = "RepoFilesSortField")]
    pub type RepoFilesSortField;

    #[wasm_bindgen(typescript_type = "RepoFilesBrowserInfo | undefined")]
    pub type RepoFilesBrowserInfoOption;

    #[wasm_bindgen(typescript_type = "RepoFilesBrowserItem[]")]
    pub type RepoFilesBrowserItemVec;

    #[wasm_bindgen(typescript_type = "RepoFilesDetailsOptions")]
    pub type RepoFilesDetailsOptions;

    #[wasm_bindgen(typescript_type = "RepoFilesDetailsInfo | undefined")]
    pub type RepoFilesDetailsInfoOption;

    #[wasm_bindgen(typescript_type = "RepoFilesMoveMode")]
    pub type RepoFilesMoveMode;

    #[wasm_bindgen(typescript_type = "RepoFilesMoveInfo | undefined")]
    pub type RepoFilesMoveInfoOption;

    #[wasm_bindgen(typescript_type = "TransfersSummary | undefined")]
    pub type TransfersSummaryOption;

    #[wasm_bindgen(typescript_type = "TransfersList | undefined")]
    pub type TransfersListOption;

    #[wasm_bindgen(typescript_type = "FileStream | undefined")]
    pub type FileStreamOption;

    #[wasm_bindgen(typescript_type = "DirPickerItem[] | undefined")]
    pub type DirPickerItemVecOption;

    #[wasm_bindgen(typescript_type = "SpaceUsage | undefined")]
    pub type SpaceUsageOption;
}

pub fn to_js<In: serde::ser::Serialize + ?Sized, Out: From<JsValue> + Into<JsValue>>(
    value: &In,
) -> Out {
    serde_wasm_bindgen::to_value(value).unwrap().into()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout", catch)]
    fn set_timeout(handler: &js_sys::Function, timeout: i32) -> Result<JsValue, JsValue>;
}

pub fn to_cb(callback: js_sys::Function) -> Box<dyn Fn() + Send + Sync + 'static> {
    let callback: Box<dyn Fn() + 'static> = Box::new(move || {
        set_timeout(&callback, 0).unwrap();
    });

    let callback: Box<dyn Fn() + Send + Sync + 'static> = unsafe {
        Box::from_raw(Box::into_raw(callback) as *mut (dyn Fn() + Send + Sync + 'static))
    };

    callback
}

#[wasm_bindgen]
pub struct WebVault {
    vault: Arc<vault_core::Vault>,
    base: Arc<vault_web_api::web_vault_base::WebVaultBase>,
    errors: Arc<WebErrors>,
}

#[wasm_bindgen]
impl WebVault {
    #[wasm_bindgen(constructor)]
    pub fn new(
        base_url: String,
        oauth2_auth_base_url: String,
        oauth2_client_id: String,
        oauth2_client_secret: String,
        oauth2_redirect_uri: String,
        browser_http_client_delegate: BrowserHttpClientDelegate,
        browser_eventstream_websocket_delegate: BrowserEventstreamWebSocketDelegate,
        storage: Storage,
    ) -> Self {
        let oauth2_config = vault_core::oauth2::OAuth2Config {
            base_url: base_url.clone(),
            auth_base_url: oauth2_auth_base_url.clone(),
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
            Box::new(BrowserSecureStorage::new(storage)),
            Box::new(BrowserRuntime::new()),
        ));

        let base = Arc::new(vault_web_api::web_vault_base::WebVaultBase::new(
            vault.clone(),
        ));

        let errors = Arc::new(WebErrors::new(vault.clone()));

        Self {
            vault,
            base,
            errors,
        }
    }
}

// proxy to base

#[wasm_bindgen]
impl WebVault {
    // subscription

    #[wasm_bindgen(js_name = unsubscribe)]
    pub fn unsubscribe(&self, id: u32) {
        self.base.unsubscribe(id);
    }

    // lifecycle

    #[wasm_bindgen(js_name = load)]
    pub fn load(&self) {
        self.base.load();
    }

    #[wasm_bindgen(js_name = logout)]
    pub fn logout(&self) {
        self.base.logout();
    }

    #[wasm_bindgen(js_name = appVisible)]
    pub fn app_visible(&self) {
        self.base.app_visible();
    }

    #[wasm_bindgen(js_name = appHidden)]
    pub fn app_hidden(&self) {
        self.base.app_hidden();
    }

    // relative_time

    #[wasm_bindgen(js_name = relativeTime)]
    pub fn relative_time(&self, value: f64, with_modifier: bool) -> RelativeTime {
        to_js(&self.base.relative_time(value, with_modifier))
    }

    // notifications

    #[wasm_bindgen(js_name = notificationsSubscribe)]
    pub fn notifications_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.notifications_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = notificationsData)]
    pub fn notifications_data(&self, id: u32) -> NotificationVecOption {
        to_js(&self.base.notifications_data(id))
    }

    #[wasm_bindgen(js_name = notificationsRemove)]
    pub fn notifications_remove(&self, notification_id: u32) {
        self.base.notifications_remove(notification_id);
    }

    #[wasm_bindgen(js_name = notificationsRemoveAfter)]
    pub fn notifications_remove_after(&self, notification_id: u32, duration_ms: u32) {
        self.base
            .notifications_remove_after(notification_id, duration_ms);
    }

    #[wasm_bindgen(js_name = notificationsRemoveAll)]
    pub fn notifications_remove_all(&self) {
        self.base.notifications_remove_all();
    }

    // dialogs

    #[wasm_bindgen(js_name = dialogsSubscribe)]
    pub fn dialogs_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.dialogs_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = dialogsData)]
    pub fn dialogs_data(&self, id: u32) -> IdVecOption {
        to_js(&self.base.dialogs_data(id))
    }

    #[wasm_bindgen(js_name = dialogsDialogSubscribe)]
    pub fn dialogs_dialog_subscribe(&self, dialog_id: u32, cb: js_sys::Function) -> u32 {
        self.base.dialogs_dialog_subscribe(dialog_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = dialogsDialogData)]
    pub fn dialogs_dialog_data(&self, id: u32) -> DialogOption {
        to_js(&self.base.dialogs_dialog_data(id))
    }

    #[wasm_bindgen(js_name = dialogsConfirm)]
    pub fn dialogs_confirm(&self, dialog_id: u32) {
        self.base.dialogs_confirm(dialog_id);
    }

    #[wasm_bindgen(js_name = dialogsCancel)]
    pub fn dialogs_cancel(&self, dialog_id: u32) {
        self.base.dialogs_cancel(dialog_id);
    }

    #[wasm_bindgen(js_name = dialogsSetInputValue)]
    pub fn dialogs_set_input_value(&self, dialog_id: u32, value: String) {
        self.base.dialogs_set_input_value(dialog_id, value);
    }

    // oauth2

    #[wasm_bindgen(js_name = oauth2StatusSubscribe)]
    pub fn oauth2_status_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.oauth2_status_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = oauth2StatusData)]
    pub fn oauth2_status_data(&self, id: u32) -> StatusOption {
        to_js(&self.base.oauth2_status_data(id))
    }

    #[wasm_bindgen(js_name = oauth2StartLoginFlow)]
    pub fn oauth2_start_login_flow(&self) -> Option<String> {
        self.base.oauth2_start_login_flow()
    }

    #[wasm_bindgen(js_name = oauth2StartLogoutFlow)]
    pub fn oauth2_start_logout_flow(&self) -> Option<String> {
        self.base.oauth2_start_logout_flow()
    }

    #[wasm_bindgen(js_name = oauth2FinishFlowUrl)]
    pub async fn oauth2_finish_flow_url(&self, url: String) -> bool {
        self.base.oauth2_finish_flow_url(url).await
    }

    // config

    #[wasm_bindgen(js_name = configGetBaseUrl)]
    pub fn config_get_base_url(&self) -> String {
        self.base.config_get_base_url()
    }

    // user

    #[wasm_bindgen(js_name = userSubscribe)]
    pub fn user_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.user_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = userData)]
    pub fn user_data(&self, id: u32) -> UserOption {
        to_js(&self.base.user_data(id))
    }

    #[wasm_bindgen(js_name = userProfilePictureLoadedSubscribe)]
    pub fn user_profile_picture_loaded_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.user_profile_picture_loaded_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = userProfilePictureLoadedData)]
    pub fn user_profile_picture_loaded_data(&self, id: u32) -> bool {
        self.base.user_profile_picture_loaded_data(id)
    }

    #[wasm_bindgen(js_name = userGetProfilePicture)]
    pub fn user_get_profile_picture(&self) -> Bytes {
        match self.base.user_get_profile_picture() {
            Some(bytes) => helpers::bytes_to_array(&bytes).into(),
            None => JsValue::UNDEFINED.into(),
        }
    }

    #[wasm_bindgen(js_name = userEnsureProfilePicture)]
    pub fn user_ensure_profile_picture(&self) {
        self.base.user_ensure_profile_picture();
    }

    // file_icon

    #[wasm_bindgen(js_name = fileIconSvg)]
    pub fn file_icon_svg(&self, props: FileIconProps) -> String {
        self.base
            .file_icon_svg(serde_wasm_bindgen::from_value(props.into()).unwrap())
    }

    // repos

    #[wasm_bindgen(js_name = reposSubscribe)]
    pub fn repos_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.repos_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = reposData)]
    pub fn repos_data(&self, id: u32) -> ReposOption {
        to_js(&self.base.repos_data(id))
    }

    #[wasm_bindgen(js_name = reposRepoSubscribe)]
    pub fn repos_repo_subscribe(&self, repo_id: String, cb: js_sys::Function) -> u32 {
        self.base.repos_repo_subscribe(repo_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = reposRepoData)]
    pub fn repos_repo_data(&self, id: u32) -> RepoInfoOption {
        to_js(&self.base.repos_repo_data(id))
    }

    #[wasm_bindgen(js_name = reposLockRepo)]
    pub fn repos_lock_repo(&self, repo_id: String) {
        self.base.repos_lock_repo(repo_id);
    }

    #[wasm_bindgen(js_name = reposTouchRepo)]
    pub fn repos_touch_repo(&self, repo_id: String) {
        self.base.repos_touch_repo(repo_id);
    }

    #[wasm_bindgen(js_name = reposSetAutoLock)]
    pub fn repos_set_auto_lock(&self, repo_id: String, auto_lock: RepoAutoLock) {
        self.base.repos_set_auto_lock(
            repo_id,
            serde_wasm_bindgen::from_value(auto_lock.into()).unwrap(),
        );
    }

    #[wasm_bindgen(js_name = reposSetDefaultAutoLock)]
    pub fn repos_set_default_auto_lock(&self, auto_lock: RepoAutoLock) {
        self.base
            .repos_set_default_auto_lock(serde_wasm_bindgen::from_value(auto_lock.into()).unwrap());
    }

    // repo_create

    #[wasm_bindgen(js_name = repoCreateCreate)]
    pub fn repo_create_create(&self) -> u32 {
        self.base.repo_create_create()
    }

    #[wasm_bindgen(js_name = repoCreateInfoSubscribe)]
    pub fn repo_create_info_subscribe(&self, create_id: u32, cb: js_sys::Function) -> u32 {
        self.base.repo_create_info_subscribe(create_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoCreateInfoData)]
    pub fn repo_create_info_data(&self, id: u32) -> RepoCreateInfoOption {
        to_js(&self.base.repo_create_info_data(id))
    }

    #[wasm_bindgen(js_name = repoCreateSetPassword)]
    pub fn repo_create_set_password(&self, create_id: u32, password: String) {
        self.base.repo_create_set_password(create_id, password);
    }

    #[wasm_bindgen(js_name = repoCreateSetSalt)]
    pub fn repo_create_set_salt(&self, create_id: u32, salt: Option<String>) {
        self.base.repo_create_set_salt(create_id, salt);
    }

    #[wasm_bindgen(js_name = repoCreateFillFromRcloneConfig)]
    pub fn repo_create_fill_from_rclone_config(&self, create_id: u32, config: String) {
        self.base
            .repo_create_fill_from_rclone_config(create_id, config);
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerShow)]
    pub fn repo_create_location_dir_picker_show(&self, create_id: u32) {
        self.base.repo_create_location_dir_picker_show(create_id);
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerClick)]
    pub fn repo_create_location_dir_picker_click(
        &self,
        create_id: u32,
        item_id: String,
        is_arrow: bool,
    ) {
        self.base
            .repo_create_location_dir_picker_click(create_id, item_id, is_arrow);
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerSelect)]
    pub fn repo_create_location_dir_picker_select(&self, create_id: u32) {
        self.base.repo_create_location_dir_picker_select(create_id);
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerCancel)]
    pub fn repo_create_location_dir_picker_cancel(&self, create_id: u32) {
        self.base.repo_create_location_dir_picker_cancel(create_id);
    }

    #[wasm_bindgen(js_name = repoCreateLocationDirPickerCreateDir)]
    pub fn repo_create_location_dir_picker_create_dir(&self, create_id: u32) {
        self.base
            .repo_create_location_dir_picker_create_dir(create_id);
    }

    #[wasm_bindgen(js_name = repoCreateCreateRepo)]
    pub fn repo_create_create_repo(&self, create_id: u32) {
        self.base.repo_create_create_repo(create_id);
    }

    #[wasm_bindgen(js_name = repoCreateDestroy)]
    pub fn repo_create_destroy(&self, create_id: u32) {
        self.base.repo_create_destroy(create_id);
    }

    // repo_unlock

    #[wasm_bindgen(js_name = repoUnlockCreate)]
    pub fn repo_unlock_create(&self, repo_id: String, options: RepoUnlockOptions) -> u32 {
        self.base.repo_unlock_create(
            repo_id,
            serde_wasm_bindgen::from_value(options.into()).unwrap(),
        )
    }

    #[wasm_bindgen(js_name = repoUnlockInfoSubscribe)]
    pub fn repo_unlock_info_subscribe(&self, unlock_id: u32, cb: js_sys::Function) -> u32 {
        self.base.repo_unlock_info_subscribe(unlock_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoUnlockInfoData)]
    pub fn repo_unlock_info_data(&self, id: u32) -> RepoUnlockInfoOption {
        to_js(&self.base.repo_unlock_info_data(id))
    }

    #[wasm_bindgen(js_name = repoUnlockUnlock)]
    pub fn repo_unlock_unlock(&self, unlock_id: u32, password: String) {
        self.base.repo_unlock_unlock(unlock_id, password);
    }

    #[wasm_bindgen(js_name = repoUnlockDestroy)]
    pub fn repo_unlock_destroy(&self, unlock_id: u32) {
        self.base.repo_unlock_destroy(unlock_id);
    }

    // repo_remove

    #[wasm_bindgen(js_name = repoRemoveCreate)]
    pub fn repo_remove_create(&self, repo_id: String) -> u32 {
        self.base.repo_remove_create(repo_id)
    }

    #[wasm_bindgen(js_name = repoRemoveInfoSubscribe)]
    pub fn repo_remove_info_subscribe(&self, remove_id: u32, cb: js_sys::Function) -> u32 {
        self.base.repo_remove_info_subscribe(remove_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoRemoveInfoData)]
    pub fn repo_remove_info_data(&self, id: u32) -> RepoRemoveInfoOption {
        to_js(&self.base.repo_remove_info_data(id))
    }

    #[wasm_bindgen(js_name = repoRemoveRemove)]
    pub async fn repo_remove_remove(&self, remove_id: u32, password: String) -> bool {
        self.base.repo_remove_remove(remove_id, password).await
    }

    #[wasm_bindgen(js_name = repoRemoveDestroy)]
    pub fn repo_remove_destroy(&self, remove_id: u32) {
        self.base.repo_remove_destroy(remove_id);
    }

    // repo_config_backup

    #[wasm_bindgen(js_name = repoConfigBackupCreate)]
    pub fn repo_config_backup_create(&self, repo_id: String) -> u32 {
        self.base.repo_config_backup_create(repo_id)
    }

    #[wasm_bindgen(js_name = repoConfigBackupInfoSubscribe)]
    pub fn repo_config_backup_info_subscribe(&self, backup_id: u32, cb: js_sys::Function) -> u32 {
        self.base
            .repo_config_backup_info_subscribe(backup_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoConfigBackupInfoData)]
    pub fn repo_config_backup_info_data(&self, id: u32) -> RepoConfigBackupInfoOption {
        to_js(&self.base.repo_config_backup_info_data(id))
    }

    #[wasm_bindgen(js_name = repoConfigBackupGenerate)]
    pub fn repo_config_backup_generate(&self, backup_id: u32, password: String) {
        self.base.repo_config_backup_generate(backup_id, password);
    }

    #[wasm_bindgen(js_name = repoConfigBackupDestroy)]
    pub fn repo_config_backup_destroy(&self, backup_id: u32) {
        self.base.repo_config_backup_destroy(backup_id);
    }

    // repo_space_usage

    #[wasm_bindgen(js_name = repoSpaceUsageCreate)]
    pub fn repo_space_usage_create(&self, repo_id: String) -> u32 {
        self.base.repo_space_usage_create(repo_id)
    }

    #[wasm_bindgen(js_name = repoSpaceUsageInfoSubscribe)]
    pub fn repo_space_usage_info_subscribe(&self, usage_id: u32, cb: js_sys::Function) -> u32 {
        self.base
            .repo_space_usage_info_subscribe(usage_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoSpaceUsageInfoData)]
    pub fn repo_space_usage_info_data(&self, id: u32) -> RepoSpaceUsageInfoOption {
        to_js(&self.base.repo_space_usage_info_data(id))
    }

    #[wasm_bindgen(js_name = repoSpaceUsageCalculate)]
    pub fn repo_space_usage_calculate(&self, usage_id: u32) {
        self.base.repo_space_usage_calculate(usage_id);
    }

    #[wasm_bindgen(js_name = repoSpaceUsageDestroy)]
    pub fn repo_space_usage_destroy(&self, usage_id: u32) {
        self.base.repo_space_usage_destroy(usage_id);
    }

    // repo_files

    #[wasm_bindgen(js_name = repoFilesFileSubscribe)]
    pub fn repo_files_file_subscribe(&self, file_id: String, cb: js_sys::Function) -> u32 {
        self.base.repo_files_file_subscribe(file_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoFilesFileData)]
    pub fn repo_files_file_data(&self, id: u32) -> RepoFileOption {
        to_js(&self.base.repo_files_file_data(id))
    }

    #[wasm_bindgen(js_name = repoFilesDeleteFile)]
    pub fn repo_files_delete_file(&self, repo_id: String, encrypted_path: String) {
        self.base.repo_files_delete_file(repo_id, encrypted_path);
    }

    #[wasm_bindgen(js_name = repoFilesRenameFile)]
    pub fn repo_files_rename_file(&self, repo_id: String, encrypted_path: String) {
        self.base.repo_files_rename_file(repo_id, encrypted_path);
    }

    #[wasm_bindgen(js_name = repoFilesEncryptName)]
    pub fn repo_files_encrypt_name(&self, repo_id: String, name: String) -> Option<String> {
        self.base.repo_files_encrypt_name(repo_id, name)
    }

    // transfers

    #[wasm_bindgen(js_name = transfersIsActiveSubscribe)]
    pub fn transfers_is_active_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.transfers_is_active_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = transfersIsActiveData)]
    pub fn transfers_is_active_data(&self, id: u32) -> bool {
        self.base.transfers_is_active_data(id)
    }

    #[wasm_bindgen(js_name = transfersSummarySubscribe)]
    pub fn transfers_summary_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.transfers_summary_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = transfersSummaryData)]
    pub fn transfers_summary_data(&self, id: u32) -> TransfersSummaryOption {
        to_js(&self.base.transfers_summary_data(id))
    }

    #[wasm_bindgen(js_name = transfersListSubscribe)]
    pub fn transfers_list_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.transfers_list_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = transfersListData)]
    pub fn transfers_list_data(&self, id: u32) -> TransfersListOption {
        to_js(&self.base.transfers_list_data(id))
    }

    #[wasm_bindgen(js_name = transfersAbort)]
    pub fn transfers_abort(&self, id: u32) {
        self.base.transfers_abort(id);
    }

    #[wasm_bindgen(js_name = transfersAbortAll)]
    pub fn transfers_abort_all(&self) {
        self.base.transfers_abort_all();
    }

    #[wasm_bindgen(js_name = transfersRetry)]
    pub fn transfers_retry(&self, id: u32) {
        self.base.transfers_retry(id);
    }

    #[wasm_bindgen(js_name = transfersRetryAll)]
    pub fn transfers_retry_all(&self) {
        self.base.transfers_retry_all();
    }

    // dir_pickers

    #[wasm_bindgen(js_name = dirPickersItemsSubscribe)]
    pub fn dir_pickers_items_subscribe(&self, picker_id: u32, cb: js_sys::Function) -> u32 {
        self.base.dir_pickers_items_subscribe(picker_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = dirPickersItemsData)]
    pub fn dir_pickers_items_data(&self, id: u32) -> DirPickerItemVecOption {
        to_js(&self.base.dir_pickers_items_data(id))
    }

    // repo_files_browsers

    #[wasm_bindgen(js_name = repoFilesBrowsersCreate)]
    pub fn repo_files_browsers_create(
        &self,
        repo_id: String,
        encrypted_path: String,
        options: RepoFilesBrowserOptions,
    ) -> u32 {
        self.base.repo_files_browsers_create(
            repo_id,
            encrypted_path,
            serde_wasm_bindgen::from_value(options.into()).unwrap(),
        )
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersDestroy)]
    pub fn repo_files_browsers_destroy(&self, browser_id: u32) {
        self.base.repo_files_browsers_destroy(browser_id);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersInfo)]
    pub fn repo_files_browsers_info(&self, browser_id: u32) -> RepoFilesBrowserInfoOption {
        to_js(&self.base.repo_files_browsers_info(browser_id))
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersInfoSubscribe)]
    pub fn repo_files_browsers_info_subscribe(&self, browser_id: u32, cb: js_sys::Function) -> u32 {
        self.base
            .repo_files_browsers_info_subscribe(browser_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersInfoData)]
    pub fn repo_files_browsers_info_data(&self, id: u32) -> RepoFilesBrowserInfoOption {
        to_js(&self.base.repo_files_browsers_info_data(id))
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersLoadFiles)]
    pub fn repo_files_browsers_load_files(&self, browser_id: u32) {
        self.base.repo_files_browsers_load_files(browser_id);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersSelectFile)]
    pub fn repo_files_browsers_select_file(
        &self,
        browser_id: u32,
        file_id: String,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.base
            .repo_files_browsers_select_file(browser_id, file_id, extend, range, force);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersSelectAll)]
    pub fn repo_files_browsers_select_all(&self, browser_id: u32) {
        self.base.repo_files_browsers_select_all(browser_id);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersClearSelection)]
    pub fn repo_files_browsers_clear_selection(&self, browser_id: u32) {
        self.base.repo_files_browsers_clear_selection(browser_id);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersSortBy)]
    pub fn repo_files_browsers_sort_by(&self, browser_id: u32, field: RepoFilesSortField) {
        self.base.repo_files_browsers_sort_by(
            browser_id,
            serde_wasm_bindgen::from_value(field.into()).unwrap(),
        );
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersCreateDir)]
    pub fn repo_files_browsers_create_dir(&self, browser_id: u32) {
        self.base.repo_files_browsers_create_dir(browser_id);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersCreateFile)]
    pub async fn repo_files_browsers_create_file(
        &self,
        browser_id: u32,
        name: String,
    ) -> Option<String> {
        self.base
            .repo_files_browsers_create_file(browser_id, name)
            .await
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersDeleteSelected)]
    pub fn repo_files_browsers_delete_selected(&self, browser_id: u32) {
        self.base.repo_files_browsers_delete_selected(browser_id);
    }

    #[wasm_bindgen(js_name = repoFilesBrowsersMoveSelected)]
    pub fn repo_files_browsers_move_selected(&self, browser_id: u32, mode: RepoFilesMoveMode) {
        self.base.repo_files_browsers_move_selected(
            browser_id,
            serde_wasm_bindgen::from_value(mode.into()).unwrap(),
        );
    }

    // repo_files_details

    #[wasm_bindgen(js_name = repoFilesDetailsCreate)]
    pub fn repo_files_details_create(
        &self,
        repo_id: String,
        encrypted_path: String,
        is_editing: bool,
        options: RepoFilesDetailsOptions,
    ) -> u32 {
        self.base.repo_files_details_create(
            repo_id,
            encrypted_path,
            is_editing,
            serde_wasm_bindgen::from_value(options.into()).unwrap(),
        )
    }

    #[wasm_bindgen(js_name = repoFilesDetailsDestroy)]
    pub fn repo_files_details_destroy(&self, details_id: u32) {
        self.base.repo_files_details_destroy(details_id);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsInfoSubscribe)]
    pub fn repo_files_details_info_subscribe(&self, details_id: u32, cb: js_sys::Function) -> u32 {
        self.base
            .repo_files_details_info_subscribe(details_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoFilesDetailsInfoData)]
    pub fn repo_files_details_info_data(&self, id: u32) -> RepoFilesDetailsInfoOption {
        to_js(&self.base.repo_files_details_info_data(id))
    }

    #[wasm_bindgen(js_name = repoFilesDetailsFileSubscribe)]
    pub fn repo_files_details_file_subscribe(&self, details_id: u32, cb: js_sys::Function) -> u32 {
        self.base
            .repo_files_details_file_subscribe(details_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoFilesDetailsFileData)]
    pub fn repo_files_details_file_data(&self, id: u32) -> RepoFileOption {
        to_js(&self.base.repo_files_details_file_data(id))
    }

    #[wasm_bindgen(js_name = repoFilesDetailsContentBytesSubscribe)]
    pub fn repo_files_details_content_bytes_subscribe(
        &self,
        details_id: u32,
        cb: js_sys::Function,
    ) -> u32 {
        self.base
            .repo_files_details_content_bytes_subscribe(details_id, to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoFilesDetailsContentBytesData)]
    pub fn repo_files_details_content_bytes_data(&self, id: u32) -> Bytes {
        match self.base.repo_files_details_content_bytes_data(id) {
            Some(bytes) => helpers::bytes_to_array(&bytes).into(),
            None => JsValue::UNDEFINED.into(),
        }
    }

    #[wasm_bindgen(js_name = repoFilesDetailsLoadFile)]
    pub fn repo_files_details_load_file(&self, details_id: u32) {
        self.base.repo_files_details_load_file(details_id);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsLoadContent)]
    pub fn repo_files_details_load_content(&self, details_id: u32) {
        self.base.repo_files_details_load_content(details_id);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsEdit)]
    pub fn repo_files_details_edit(&self, details_id: u32) {
        self.base.repo_files_details_edit(details_id);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsEditCancel)]
    pub fn repo_files_details_edit_cancel(&self, details_id: u32) {
        self.base.repo_files_details_edit_cancel(details_id);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsSetContent)]
    pub fn repo_files_details_set_content(&self, details_id: u32, content: Vec<u8>) {
        self.base
            .repo_files_details_set_content(details_id, content);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsSave)]
    pub fn repo_files_details_save(&self, details_id: u32) {
        self.base.repo_files_details_save(details_id);
    }

    #[wasm_bindgen(js_name = repoFilesDetailsDelete)]
    pub fn repo_files_details_delete(&self, details_id: u32) {
        self.base.repo_files_details_delete(details_id);
    }

    // repo_files_move

    #[wasm_bindgen(js_name = repoFilesMoveInfoSubscribe)]
    pub fn repo_files_move_info_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.repo_files_move_info_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = repoFilesMoveInfoData)]
    pub fn repo_files_move_info_data(&self, id: u32) -> RepoFilesMoveInfoOption {
        to_js(&self.base.repo_files_move_info_data(id))
    }

    #[wasm_bindgen(js_name = repoFilesMoveDirPickerClick)]
    pub fn repo_files_move_dir_picker_click(&self, item_id: String, is_arrow: bool) {
        self.base
            .repo_files_move_dir_picker_click(item_id, is_arrow);
    }

    #[wasm_bindgen(js_name = repoFilesMoveMoveFiles)]
    pub fn repo_files_move_move_files(&self) {
        self.base.repo_files_move_move_files();
    }

    #[wasm_bindgen(js_name = repoFilesMoveCancel)]
    pub fn repo_files_move_cancel(&self) {
        self.base.repo_files_move_cancel();
    }

    #[wasm_bindgen(js_name = repoFilesMoveCreateDir)]
    pub fn repo_files_move_create_dir(&self) {
        self.base.repo_files_move_create_dir();
    }

    // space_usage

    #[wasm_bindgen(js_name = spaceUsageSubscribe)]
    pub fn space_usage_subscribe(&self, cb: js_sys::Function) -> u32 {
        self.base.space_usage_subscribe(to_cb(cb))
    }

    #[wasm_bindgen(js_name = spaceUsageData)]
    pub fn space_usage_data(&self, id: u32) -> SpaceUsageOption {
        to_js(&self.base.space_usage_data(id))
    }
}

// browser integration

#[wasm_bindgen]
impl WebVault {
    // repo_files

    async fn repo_file_reader_to_file_stream(
        &self,
        file_reader: Result<
            vault_core::repo_files_read::state::RepoFileReader,
            vault_core::repo_files_read::errors::GetFilesReaderError,
        >,
        force_blob: bool,
        abort_signal: Option<AbortSignal>,
    ) -> FileStreamOption {
        let reader = match file_reader {
            Ok(reader) => reader,
            Err(err) => {
                self.errors.handle_error(err);

                return JsValue::UNDEFINED.into();
            }
        };

        let (transfer_id, file_reader) = self.vault.clone().transfers_download_reader(reader);

        let reader = match abort_signal {
            Some(abort_signal) => helpers::transfers_download_reader_abort_signal(
                self.vault.clone(),
                file_reader.reader,
                transfer_id,
                abort_signal,
            ),
            None => file_reader.reader,
        };

        let file_stream = match helpers::reader_to_file_stream(
            &file_reader.name.0,
            reader,
            file_reader.size,
            file_reader.content_type.as_deref(),
            force_blob,
        )
        .await
        {
            Ok(file_stream) => file_stream,
            Err(err) => {
                self.errors.handle_error(err);

                return JsValue::UNDEFINED.into();
            }
        };

        FileStreamOption::from(file_stream)
    }

    #[wasm_bindgen(js_name = repoFilesGetFileStream)]
    pub async fn repo_files_get_file_stream(
        &self,
        repo_id: String,
        encrypted_path: String,
        force_blob: bool,
    ) -> FileStreamOption {
        self.repo_file_reader_to_file_stream(
            match self
                .vault
                .clone()
                .repo_files_get_file_reader(&RepoId(repo_id), &EncryptedPath(encrypted_path))
            {
                Ok(provider) => provider.reader().await,
                Err(err) => Err(err),
            },
            force_blob,
            None,
        )
        .await
    }

    // transfers

    #[wasm_bindgen(js_name = transfersUpload)]
    pub async fn transfers_upload(
        &self,
        repo_id: String,
        encrypted_parent_path: String,
        name: String,
        file: FileOrBlob,
    ) -> RepoFilesUploadResultOption {
        let uploadable = Box::new(BrowserUploadable::from_value(file.into()).unwrap());

        let (_, create_future) = self.vault.transfers_upload(
            RepoId(repo_id),
            EncryptedPath(encrypted_parent_path),
            transfers::state::TransferUploadRelativeName(name),
            uploadable,
        );

        let future = match create_future.await {
            Ok(future) => future,
            Err(err) => {
                // create transfer errors have to be displayed
                self.errors.handle_error(err);

                return JsValue::UNDEFINED.into();
            }
        };

        match future.await {
            Ok(res) => to_js(&dto::RepoFilesUploadResult::from(res)),
            Err(_) => {
                // transfer errors are displayed in transfers component
                JsValue::UNDEFINED.into()
            }
        }
    }

    // repo_files_browsers

    #[wasm_bindgen(js_name = repoFilesBrowsersGetSelectedStream)]
    pub async fn repo_files_browsers_get_selected_stream(
        &self,
        browser_id: u32,
        force_blob: bool,
    ) -> FileStreamOption {
        self.repo_file_reader_to_file_stream(
            match self
                .vault
                .clone()
                .repo_files_browsers_get_selected_reader(browser_id)
            {
                Ok(provider) => provider.reader().await,
                Err(err) => Err(err),
            },
            force_blob,
            None,
        )
        .await
    }

    // repo_files_details

    #[wasm_bindgen(js_name = repoFilesDetailsGetFileStream)]
    pub async fn repo_files_details_get_file_stream(
        &self,
        details_id: u32,
        force_blob: bool,
        abort_signal: AbortSignal,
    ) -> FileStreamOption {
        self.repo_file_reader_to_file_stream(
            match self
                .vault
                .clone()
                .repo_files_details_get_file_reader(details_id)
                .await
            {
                Ok(provider) => provider.reader().await,
                Err(err) => Err(err),
            },
            force_blob,
            Some(abort_signal),
        )
        .await
    }
}
