use std::{path::PathBuf, sync::Arc, time::Duration};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{
        header::{self, CONTENT_TYPE},
        StatusCode,
    },
    response::{sse::Event, IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use data_encoding::BASE64;
use futures::{future::BoxFuture, FutureExt, Stream, StreamExt};
use serde::Deserialize;
use thiserror::Error;

use vault_core::{
    common::state::SizeInfo,
    repo_files_read,
    transfers::{
        self, downloadable,
        errors::{DownloadableError, TransferError},
    },
    types::{EncryptedPath, RepoId},
    user_error::StringUserError,
    utils::reader_stream::ReaderStream,
};
use vault_crypto::constants::BLOCK_SIZE;
use vault_native::transfers::{
    file_uploadable::FileUploadable, pick_file_downloadable::PickFileDownloadable,
    temp_file_downloadable::TempFileDownloadable,
};
use vault_web_api::{dto, web_vault_base::WebVaultBase};

use crate::{
    app_state::AppState,
    callbacks::CallbackId,
    extract::{ExtractBase, ExtractCallbacks, ExtractSessions},
    upload_helper,
};

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct ApiError(String);

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.clone()).into_response()
    }
}

pub fn register_routes(router: Router<AppState>) -> Router<AppState> {
    router
        //
        .route("/oauth2callback", get(oauth2_callback))
        //
        .route("/session", get(session))
        //
        .route("/WebVault/unsubscribe", post(unsubscribe))
        .route("/WebVault/load", post(load))
        .route("/WebVault/logout", post(logout))
        .route("/WebVault/appVisible", post(app_visible))
        .route("/WebVault/appHidden", post(app_hidden))
        .route("/WebVault/relativeTime", post(relative_time))
        .route(
            "/WebVault/notificationsSubscribe",
            post(notifications_subscribe),
        )
        .route("/WebVault/notificationsData", post(notifications_data))
        .route("/WebVault/notificationsRemove", post(notifications_remove))
        .route(
            "/WebVault/notificationsRemoveAfter",
            post(notifications_remove_after),
        )
        .route(
            "/WebVault/notificationsRemoveAll",
            post(notifications_remove_all),
        )
        .route("/WebVault/dialogsSubscribe", post(dialogs_subscribe))
        .route("/WebVault/dialogsData", post(dialogs_data))
        .route(
            "/WebVault/dialogsDialogSubscribe",
            post(dialogs_dialog_subscribe),
        )
        .route("/WebVault/dialogsDialogData", post(dialogs_dialog_data))
        .route("/WebVault/dialogsConfirm", post(dialogs_confirm))
        .route("/WebVault/dialogsCancel", post(dialogs_cancel))
        .route(
            "/WebVault/dialogsSetInputValue",
            post(dialogs_set_input_value),
        )
        .route(
            "/WebVault/oauth2StatusSubscribe",
            post(oauth2_status_subscribe),
        )
        .route("/WebVault/oauth2StatusData", post(oauth2_status_data))
        .route(
            "/WebVault/oauth2StartLoginFlow",
            post(oauth2_start_login_flow),
        )
        .route(
            "/WebVault/oauth2StartLogoutFlow",
            post(oauth2_start_logout_flow),
        )
        .route(
            "/WebVault/oauth2FinishFlowUrl",
            post(oauth2_finish_flow_url),
        )
        .route("/WebVault/configGetBaseUrl", post(config_get_base_url))
        .route("/WebVault/userSubscribe", post(user_subscribe))
        .route("/WebVault/userData", post(user_data))
        .route(
            "/WebVault/userProfilePictureLoadedSubscribe",
            post(user_profile_picture_loaded_subscribe),
        )
        .route(
            "/WebVault/userProfilePictureLoadedData",
            post(user_profile_picture_loaded_data),
        )
        .route(
            "/WebVault/userGetProfilePicture",
            post(user_get_profile_picture),
        )
        .route(
            "/WebVault/userEnsureProfilePicture",
            post(user_ensure_profile_picture),
        )
        .route("/WebVault/fileIconSvg", post(file_icon_svg))
        .route("/WebVault/reposSubscribe", post(repos_subscribe))
        .route("/WebVault/reposData", post(repos_data))
        .route("/WebVault/reposRepoSubscribe", post(repos_repo_subscribe))
        .route("/WebVault/reposRepoData", post(repos_repo_data))
        .route("/WebVault/reposLockRepo", post(repos_lock_repo))
        .route("/WebVault/reposTouchRepo", post(repos_touch_repo))
        .route("/WebVault/reposSetAutoLock", post(repos_set_auto_lock))
        .route(
            "/WebVault/reposSetDefaultAutoLock",
            post(repos_set_default_auto_lock),
        )
        .route("/WebVault/repoCreateCreate", post(repo_create_create))
        .route(
            "/WebVault/repoCreateInfoSubscribe",
            post(repo_create_info_subscribe),
        )
        .route("/WebVault/repoCreateInfoData", post(repo_create_info_data))
        .route(
            "/WebVault/repoCreateSetPassword",
            post(repo_create_set_password),
        )
        .route("/WebVault/repoCreateSetSalt", post(repo_create_set_salt))
        .route(
            "/WebVault/repoCreateFillFromRcloneConfig",
            post(repo_create_fill_from_rclone_config),
        )
        .route(
            "/WebVault/repoCreateLocationDirPickerShow",
            post(repo_create_location_dir_picker_show),
        )
        .route(
            "/WebVault/repoCreateLocationDirPickerClick",
            post(repo_create_location_dir_picker_click),
        )
        .route(
            "/WebVault/repoCreateLocationDirPickerSelect",
            post(repo_create_location_dir_picker_select),
        )
        .route(
            "/WebVault/repoCreateLocationDirPickerCancel",
            post(repo_create_location_dir_picker_cancel),
        )
        .route(
            "/WebVault/repoCreateLocationDirPickerCreateDir",
            post(repo_create_location_dir_picker_create_dir),
        )
        .route(
            "/WebVault/repoCreateCreateRepo",
            post(repo_create_create_repo),
        )
        .route("/WebVault/repoCreateDestroy", post(repo_create_destroy))
        .route("/WebVault/repoUnlockCreate", post(repo_unlock_create))
        .route(
            "/WebVault/repoUnlockInfoSubscribe",
            post(repo_unlock_info_subscribe),
        )
        .route("/WebVault/repoUnlockInfoData", post(repo_unlock_info_data))
        .route("/WebVault/repoUnlockUnlock", post(repo_unlock_unlock))
        .route("/WebVault/repoUnlockDestroy", post(repo_unlock_destroy))
        .route("/WebVault/repoRemoveCreate", post(repo_remove_create))
        .route(
            "/WebVault/repoRemoveInfoSubscribe",
            post(repo_remove_info_subscribe),
        )
        .route("/WebVault/repoRemoveInfoData", post(repo_remove_info_data))
        .route("/WebVault/repoRemoveRemove", post(repo_remove_remove))
        .route("/WebVault/repoRemoveDestroy", post(repo_remove_destroy))
        .route(
            "/WebVault/repoConfigBackupCreate",
            post(repo_config_backup_create),
        )
        .route(
            "/WebVault/repoConfigBackupInfoSubscribe",
            post(repo_config_backup_info_subscribe),
        )
        .route(
            "/WebVault/repoConfigBackupInfoData",
            post(repo_config_backup_info_data),
        )
        .route(
            "/WebVault/repoConfigBackupGenerate",
            post(repo_config_backup_generate),
        )
        .route(
            "/WebVault/repoConfigBackupDestroy",
            post(repo_config_backup_destroy),
        )
        .route(
            "/WebVault/repoSpaceUsageCreate",
            post(repo_space_usage_create),
        )
        .route(
            "/WebVault/repoSpaceUsageInfoSubscribe",
            post(repo_space_usage_info_subscribe),
        )
        .route(
            "/WebVault/repoSpaceUsageInfoData",
            post(repo_space_usage_info_data),
        )
        .route(
            "/WebVault/repoSpaceUsageCalculate",
            post(repo_space_usage_calculate),
        )
        .route(
            "/WebVault/repoSpaceUsageDestroy",
            post(repo_space_usage_destroy),
        )
        .route(
            "/WebVault/repoFilesFileSubscribe",
            post(repo_files_file_subscribe),
        )
        .route("/WebVault/repoFilesFileData", post(repo_files_file_data))
        .route(
            "/WebVault/repoFilesDeleteFile",
            post(repo_files_delete_file),
        )
        .route(
            "/WebVault/repoFilesRenameFile",
            post(repo_files_rename_file),
        )
        .route(
            "/WebVault/repoFilesEncryptName",
            post(repo_files_encrypt_name),
        )
        .route(
            "/WebVault/transfersIsActiveSubscribe",
            post(transfers_is_active_subscribe),
        )
        .route(
            "/WebVault/transfersIsActiveData",
            post(transfers_is_active_data),
        )
        .route(
            "/WebVault/transfersSummarySubscribe",
            post(transfers_summary_subscribe),
        )
        .route(
            "/WebVault/transfersSummaryData",
            post(transfers_summary_data),
        )
        .route(
            "/WebVault/transfersListSubscribe",
            post(transfers_list_subscribe),
        )
        .route("/WebVault/transfersListData", post(transfers_list_data))
        .route("/WebVault/transfersAbort", post(transfers_abort))
        .route("/WebVault/transfersAbortAll", post(transfers_abort_all))
        .route("/WebVault/transfersRetry", post(transfers_retry))
        .route("/WebVault/transfersRetryAll", post(transfers_retry_all))
        .route("/WebVault/transfersOpen", post(transfers_open))
        .route(
            "/WebVault/dirPickersItemsSubscribe",
            post(dir_pickers_items_subscribe),
        )
        .route(
            "/WebVault/dirPickersItemsData",
            post(dir_pickers_items_data),
        )
        .route(
            "/WebVault/repoFilesBrowsersCreate",
            post(repo_files_browsers_create),
        )
        .route(
            "/WebVault/repoFilesBrowsersDestroy",
            post(repo_files_browsers_destroy),
        )
        .route(
            "/WebVault/repoFilesBrowsersInfo",
            post(repo_files_browsers_info),
        )
        .route(
            "/WebVault/repoFilesBrowsersInfoSubscribe",
            post(repo_files_browsers_info_subscribe),
        )
        .route(
            "/WebVault/repoFilesBrowsersInfoData",
            post(repo_files_browsers_info_data),
        )
        .route(
            "/WebVault/repoFilesBrowsersLoadFiles",
            post(repo_files_browsers_load_files),
        )
        .route(
            "/WebVault/repoFilesBrowsersSelectFile",
            post(repo_files_browsers_select_file),
        )
        .route(
            "/WebVault/repoFilesBrowsersSelectAll",
            post(repo_files_browsers_select_all),
        )
        .route(
            "/WebVault/repoFilesBrowsersClearSelection",
            post(repo_files_browsers_clear_selection),
        )
        .route(
            "/WebVault/repoFilesBrowsersSortBy",
            post(repo_files_browsers_sort_by),
        )
        .route(
            "/WebVault/repoFilesBrowsersCreateDir",
            post(repo_files_browsers_create_dir),
        )
        .route(
            "/WebVault/repoFilesBrowsersCreateFile",
            post(repo_files_browsers_create_file),
        )
        .route(
            "/WebVault/repoFilesBrowsersDeleteSelected",
            post(repo_files_browsers_delete_selected),
        )
        .route(
            "/WebVault/repoFilesBrowsersMoveSelected",
            post(repo_files_browsers_move_selected),
        )
        .route(
            "/WebVault/repoFilesDetailsCreate",
            post(repo_files_details_create),
        )
        .route(
            "/WebVault/repoFilesDetailsDestroy",
            post(repo_files_details_destroy),
        )
        .route(
            "/WebVault/repoFilesDetailsInfoSubscribe",
            post(repo_files_details_info_subscribe),
        )
        .route(
            "/WebVault/repoFilesDetailsInfoData",
            post(repo_files_details_info_data),
        )
        .route(
            "/WebVault/repoFilesDetailsFileSubscribe",
            post(repo_files_details_file_subscribe),
        )
        .route(
            "/WebVault/repoFilesDetailsFileData",
            post(repo_files_details_file_data),
        )
        .route(
            "/WebVault/repoFilesDetailsContentBytesSubscribe",
            post(repo_files_details_content_bytes_subscribe),
        )
        .route(
            "/WebVault/repoFilesDetailsContentBytesData",
            post(repo_files_details_content_bytes_data),
        )
        .route(
            "/WebVault/repoFilesDetailsLoadFile",
            post(repo_files_details_load_file),
        )
        .route(
            "/WebVault/repoFilesDetailsLoadContent",
            post(repo_files_details_load_content),
        )
        .route(
            "/WebVault/repoFilesDetailsEdit",
            post(repo_files_details_edit),
        )
        .route(
            "/WebVault/repoFilesDetailsEditCancel",
            post(repo_files_details_edit_cancel),
        )
        .route(
            "/WebVault/repoFilesDetailsSetContent",
            post(repo_files_details_set_content),
        )
        .route(
            "/WebVault/repoFilesDetailsSave",
            post(repo_files_details_save),
        )
        .route(
            "/WebVault/repoFilesDetailsDelete",
            post(repo_files_details_delete),
        )
        .route(
            "/WebVault/repoFilesMoveInfoSubscribe",
            post(repo_files_move_info_subscribe),
        )
        .route(
            "/WebVault/repoFilesMoveInfoData",
            post(repo_files_move_info_data),
        )
        .route(
            "/WebVault/repoFilesMoveDirPickerClick",
            post(repo_files_move_dir_picker_click),
        )
        .route(
            "/WebVault/repoFilesMoveMoveFiles",
            post(repo_files_move_move_files),
        )
        .route(
            "/WebVault/repoFilesMoveCancel",
            post(repo_files_move_cancel),
        )
        .route(
            "/WebVault/repoFilesMoveCreateDir",
            post(repo_files_move_create_dir),
        )
        .route("/WebVault/spaceUsageSubscribe", post(space_usage_subscribe))
        .route("/WebVault/spaceUsageData", post(space_usage_data))
        //
        .route("/WebVault/login", post(login))
        .route("/WebVault/oauth2Logout", post(oauth2_logout))
        .route("/WebVault/repoFilesOpenFile", post(repo_files_open_file))
        .route(
            "/WebVault/repoFilesDownloadFile",
            post(repo_files_download_file),
        )
        .route(
            "/WebVault/repoFilesUploadFile",
            post(repo_files_upload_file),
        )
        .route("/WebVault/repoFilesUploadDir", post(repo_files_upload_dir))
        .route(
            "/WebVault/repoFilesUploadPaths",
            post(repo_files_upload_paths),
        )
        .route(
            "/WebVault/repoFilesDetailsGetFileStream",
            get(repo_files_details_get_file_stream),
        )
        .route(
            "/WebVault/repoFilesBrowsersDownloadSelected",
            post(repo_files_browsers_download_selected),
        )
}

pub async fn session(
    ExtractSessions(sessions): ExtractSessions,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let stream = sessions
        .create_session()
        .map(|message| Event::default().json_data(message));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

// desktop

pub async fn oauth2_callback(
    ExtractBase(base): ExtractBase,
    parts: axum::http::request::Parts,
) -> Response {
    let url = format!("http://host{}", parts.uri.to_string());

    base.clone().spawn(|_| {
        async move {
            let _ = base.oauth2_finish_flow_url(url).await;
        }
        .boxed()
    });

    (
        [(CONTENT_TYPE, "text/html; charset=utf-8")],
        "<center><h1>You can now close this page.</h1></center>",
    )
        .into_response()
}

pub async fn login(ExtractBase(base): ExtractBase) {
    if let Some(url) = base.oauth2_start_login_flow() {
        if let Err(err) = open::that(&url) {
            log::warn!("Failed to open OAuth2 login url: {:?}", err);
            log::info!("Open the following URL: {}", url);
        }
    }
}

pub async fn oauth2_logout(ExtractBase(base): ExtractBase) {
    if let Some(url) = base.oauth2_start_logout_flow() {
        if let Err(err) = open::that(&url) {
            log::warn!("Failed to open OAuth2 logout url: {:?}", err);
            log::info!("Open the following URL: {}", url);
        }
    }
}

// subscription

pub async fn unsubscribe(ExtractBase(base): ExtractBase, Json((id,)): Json<(u32,)>) {
    base.unsubscribe(id);
}

// lifecycle

pub async fn load(ExtractBase(base): ExtractBase) {
    base.load();
}

pub async fn logout(ExtractBase(base): ExtractBase) {
    base.logout();
}

pub async fn app_visible(ExtractBase(base): ExtractBase) {
    base.app_visible();
}

pub async fn app_hidden(ExtractBase(base): ExtractBase) {
    base.app_hidden();
}

// relative_time

pub async fn relative_time(
    ExtractBase(base): ExtractBase,
    Json((value, with_modifier)): Json<(f64, bool)>,
) -> Json<dto::RelativeTime> {
    Json(base.relative_time(value, with_modifier))
}

// notifications

pub async fn notifications_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.notifications_subscribe(callbacks.cb(cb)))
}

pub async fn notifications_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<Vec<dto::Notification>>> {
    Json(base.notifications_data(id))
}

pub async fn notifications_remove(
    ExtractBase(base): ExtractBase,
    Json((notification_id,)): Json<(u32,)>,
) {
    base.notifications_remove(notification_id);
}

pub async fn notifications_remove_after(
    ExtractBase(base): ExtractBase,
    Json((notification_id, duration_ms)): Json<(u32, u32)>,
) {
    base.notifications_remove_after(notification_id, duration_ms);
}

pub async fn notifications_remove_all(ExtractBase(base): ExtractBase) {
    base.notifications_remove_all();
}

// dialogs

pub async fn dialogs_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.dialogs_subscribe(callbacks.cb(cb)))
}

pub async fn dialogs_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<Vec<u32>>> {
    Json(base.dialogs_data(id))
}

pub async fn dialogs_dialog_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((dialog_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.dialogs_dialog_subscribe(dialog_id, callbacks.cb(cb)))
}

pub async fn dialogs_dialog_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::Dialog>> {
    Json(base.dialogs_dialog_data(id))
}

pub async fn dialogs_confirm(ExtractBase(base): ExtractBase, Json((dialog_id,)): Json<(u32,)>) {
    base.dialogs_confirm(dialog_id);
}

pub async fn dialogs_cancel(ExtractBase(base): ExtractBase, Json((dialog_id,)): Json<(u32,)>) {
    base.dialogs_cancel(dialog_id);
}

pub async fn dialogs_set_input_value(
    ExtractBase(base): ExtractBase,
    Json((dialog_id, value)): Json<(u32, String)>,
) {
    base.dialogs_set_input_value(dialog_id, value);
}

// oauth2

pub async fn oauth2_status_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.oauth2_status_subscribe(callbacks.cb(cb)))
}

pub async fn oauth2_status_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::Status>> {
    Json(base.oauth2_status_data(id))
}

pub async fn oauth2_start_login_flow(ExtractBase(base): ExtractBase) -> Json<Option<String>> {
    Json(base.oauth2_start_login_flow())
}

pub async fn oauth2_start_logout_flow(ExtractBase(base): ExtractBase) -> Json<Option<String>> {
    Json(base.oauth2_start_logout_flow())
}

pub async fn oauth2_finish_flow_url(
    ExtractBase(base): ExtractBase,
    Json((url,)): Json<(String,)>,
) -> Json<bool> {
    Json(base.oauth2_finish_flow_url(url).await)
}

// config

pub async fn config_get_base_url(ExtractBase(base): ExtractBase) -> Json<String> {
    Json(base.config_get_base_url())
}

// user

pub async fn user_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.user_subscribe(callbacks.cb(cb)))
}

pub async fn user_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::User>> {
    Json(base.user_data(id))
}

pub async fn user_profile_picture_loaded_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.user_profile_picture_loaded_subscribe(callbacks.cb(cb)))
}

pub async fn user_profile_picture_loaded_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<bool> {
    Json(base.user_profile_picture_loaded_data(id))
}

pub async fn user_get_profile_picture(ExtractBase(base): ExtractBase) -> Response {
    match base.user_get_profile_picture() {
        Some(bytes) => BASE64.encode(&bytes).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

pub async fn user_ensure_profile_picture(ExtractBase(base): ExtractBase) {
    base.user_ensure_profile_picture();
}

// file_icon

pub async fn file_icon_svg(
    ExtractBase(base): ExtractBase,
    Json((props,)): Json<(dto::FileIconProps,)>,
) -> Json<String> {
    Json(base.file_icon_svg(props))
}

// repos

pub async fn repos_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.repos_subscribe(callbacks.cb(cb)))
}

pub async fn repos_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::Repos>> {
    Json(base.repos_data(id))
}

pub async fn repos_repo_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((repo_id, cb)): Json<(String, CallbackId)>,
) -> Json<u32> {
    Json(base.repos_repo_subscribe(repo_id, callbacks.cb(cb)))
}

pub async fn repos_repo_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoInfo>> {
    Json(base.repos_repo_data(id))
}

pub async fn repos_lock_repo(ExtractBase(base): ExtractBase, Json((repo_id,)): Json<(String,)>) {
    base.repos_lock_repo(repo_id);
}

pub async fn repos_touch_repo(ExtractBase(base): ExtractBase, Json((repo_id,)): Json<(String,)>) {
    base.repos_touch_repo(repo_id);
}

pub async fn repos_set_auto_lock(
    ExtractBase(base): ExtractBase,
    Json((repo_id, auto_lock)): Json<(String, dto::RepoAutoLock)>,
) {
    base.repos_set_auto_lock(repo_id, auto_lock);
}

pub async fn repos_set_default_auto_lock(
    ExtractBase(base): ExtractBase,
    Json((auto_lock,)): Json<(dto::RepoAutoLock,)>,
) {
    base.repos_set_default_auto_lock(auto_lock);
}

// repo_create

pub async fn repo_create_create(ExtractBase(base): ExtractBase) -> Json<u32> {
    Json(base.repo_create_create())
}

pub async fn repo_create_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((create_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_create_info_subscribe(create_id, callbacks.cb(cb)))
}

pub async fn repo_create_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoCreateInfo>> {
    Json(base.repo_create_info_data(id))
}

pub async fn repo_create_set_password(
    ExtractBase(base): ExtractBase,
    Json((create_id, password)): Json<(u32, String)>,
) {
    base.repo_create_set_password(create_id, password);
}

pub async fn repo_create_set_salt(
    ExtractBase(base): ExtractBase,
    Json((create_id, salt)): Json<(u32, Option<String>)>,
) {
    base.repo_create_set_salt(create_id, salt);
}

pub async fn repo_create_fill_from_rclone_config(
    ExtractBase(base): ExtractBase,
    Json((create_id, config)): Json<(u32, String)>,
) {
    base.repo_create_fill_from_rclone_config(create_id, config);
}

pub async fn repo_create_location_dir_picker_show(
    ExtractBase(base): ExtractBase,
    Json((create_id,)): Json<(u32,)>,
) {
    base.repo_create_location_dir_picker_show(create_id);
}

pub async fn repo_create_location_dir_picker_click(
    ExtractBase(base): ExtractBase,
    Json((create_id, item_id, is_arrow)): Json<(u32, String, bool)>,
) {
    base.repo_create_location_dir_picker_click(create_id, item_id, is_arrow);
}

pub async fn repo_create_location_dir_picker_select(
    ExtractBase(base): ExtractBase,
    Json((create_id,)): Json<(u32,)>,
) {
    base.repo_create_location_dir_picker_select(create_id);
}

pub async fn repo_create_location_dir_picker_cancel(
    ExtractBase(base): ExtractBase,
    Json((create_id,)): Json<(u32,)>,
) {
    base.repo_create_location_dir_picker_cancel(create_id);
}

pub async fn repo_create_location_dir_picker_create_dir(
    ExtractBase(base): ExtractBase,
    Json((create_id,)): Json<(u32,)>,
) {
    base.repo_create_location_dir_picker_create_dir(create_id);
}

pub async fn repo_create_create_repo(
    ExtractBase(base): ExtractBase,
    Json((create_id,)): Json<(u32,)>,
) {
    base.repo_create_create_repo(create_id);
}

pub async fn repo_create_destroy(ExtractBase(base): ExtractBase, Json((create_id,)): Json<(u32,)>) {
    base.repo_create_destroy(create_id);
}

// repo_unlock

pub async fn repo_unlock_create(
    ExtractBase(base): ExtractBase,
    Json((repo_id, options)): Json<(String, dto::RepoUnlockOptions)>,
) -> Json<u32> {
    Json(base.repo_unlock_create(repo_id, options))
}

pub async fn repo_unlock_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((unlock_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_unlock_info_subscribe(unlock_id, callbacks.cb(cb)))
}

pub async fn repo_unlock_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoUnlockInfo>> {
    Json(base.repo_unlock_info_data(id))
}

pub async fn repo_unlock_unlock(
    ExtractBase(base): ExtractBase,
    Json((unlock_id, password)): Json<(u32, String)>,
) {
    base.repo_unlock_unlock(unlock_id, password);
}

pub async fn repo_unlock_destroy(ExtractBase(base): ExtractBase, Json((unlock_id,)): Json<(u32,)>) {
    base.repo_unlock_destroy(unlock_id);
}

// repo_remove

pub async fn repo_remove_create(
    ExtractBase(base): ExtractBase,
    Json((repo_id,)): Json<(String,)>,
) -> Json<u32> {
    Json(base.repo_remove_create(repo_id))
}

pub async fn repo_remove_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((remove_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_remove_info_subscribe(remove_id, callbacks.cb(cb)))
}

pub async fn repo_remove_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoRemoveInfo>> {
    Json(base.repo_remove_info_data(id))
}

pub async fn repo_remove_remove(
    ExtractBase(base): ExtractBase,
    Json((remove_id, password)): Json<(u32, String)>,
) -> Json<bool> {
    Json(base.repo_remove_remove(remove_id, password).await)
}

pub async fn repo_remove_destroy(ExtractBase(base): ExtractBase, Json((remove_id,)): Json<(u32,)>) {
    base.repo_remove_destroy(remove_id);
}

// repo_config_backup

pub async fn repo_config_backup_create(
    ExtractBase(base): ExtractBase,
    Json((repo_id,)): Json<(String,)>,
) -> Json<u32> {
    Json(base.repo_config_backup_create(repo_id))
}

pub async fn repo_config_backup_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((backup_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_config_backup_info_subscribe(backup_id, callbacks.cb(cb)))
}

pub async fn repo_config_backup_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoConfigBackupInfo>> {
    Json(base.repo_config_backup_info_data(id))
}

pub async fn repo_config_backup_generate(
    ExtractBase(base): ExtractBase,
    Json((backup_id, password)): Json<(u32, String)>,
) {
    base.repo_config_backup_generate(backup_id, password);
}

pub async fn repo_config_backup_destroy(
    ExtractBase(base): ExtractBase,
    Json((backup_id,)): Json<(u32,)>,
) {
    base.repo_config_backup_destroy(backup_id);
}

// repo_space_usage

pub async fn repo_space_usage_create(
    ExtractBase(base): ExtractBase,
    Json((repo_id,)): Json<(String,)>,
) -> Json<u32> {
    Json(base.repo_space_usage_create(repo_id))
}

pub async fn repo_space_usage_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((usage_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_space_usage_info_subscribe(usage_id, callbacks.cb(cb)))
}

pub async fn repo_space_usage_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoSpaceUsageInfo>> {
    Json(base.repo_space_usage_info_data(id))
}

pub async fn repo_space_usage_calculate(
    ExtractBase(base): ExtractBase,
    Json((usage_id,)): Json<(u32,)>,
) {
    base.repo_space_usage_calculate(usage_id);
}

pub async fn repo_space_usage_destroy(
    ExtractBase(base): ExtractBase,
    Json((usage_id,)): Json<(u32,)>,
) {
    base.repo_space_usage_destroy(usage_id);
}

// repo_files

pub async fn repo_files_file_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((file_id, cb)): Json<(String, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_files_file_subscribe(file_id, callbacks.cb(cb)))
}

pub async fn repo_files_file_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoFile>> {
    Json(base.repo_files_file_data(id))
}

pub async fn repo_files_delete_file(
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path)): Json<(String, String)>,
) {
    base.repo_files_delete_file(repo_id, encrypted_path);
}

pub async fn repo_files_rename_file(
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path)): Json<(String, String)>,
) {
    base.repo_files_rename_file(repo_id, encrypted_path);
}

pub async fn repo_files_encrypt_name(
    ExtractBase(base): ExtractBase,
    Json((repo_id, name)): Json<(String, String)>,
) -> Json<Option<String>> {
    Json(base.repo_files_encrypt_name(repo_id, name))
}

// transfers

pub async fn transfers_is_active_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.transfers_is_active_subscribe(callbacks.cb(cb)))
}

pub async fn transfers_is_active_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<bool> {
    Json(base.transfers_is_active_data(id))
}

pub async fn transfers_summary_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.transfers_summary_subscribe(callbacks.cb(cb)))
}

pub async fn transfers_summary_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::TransfersSummary>> {
    Json(base.transfers_summary_data(id))
}

pub async fn transfers_list_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.transfers_list_subscribe(callbacks.cb(cb)))
}

pub async fn transfers_list_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::TransfersList>> {
    Json(base.transfers_list_data(id))
}

pub async fn transfers_abort(ExtractBase(base): ExtractBase, Json((id,)): Json<(u32,)>) {
    base.transfers_abort(id);
}

pub async fn transfers_abort_all(ExtractBase(base): ExtractBase) {
    base.transfers_abort_all();
}

pub async fn transfers_retry(ExtractBase(base): ExtractBase, Json((id,)): Json<(u32,)>) {
    base.transfers_retry(id);
}

pub async fn transfers_retry_all(ExtractBase(base): ExtractBase) {
    base.transfers_retry_all();
}

pub async fn transfers_open(ExtractBase(base): ExtractBase, Json((id,)): Json<(u32,)>) {
    base.transfers_open(id);
}

// dir_pickers

pub async fn dir_pickers_items_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((picker_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.dir_pickers_items_subscribe(picker_id, callbacks.cb(cb)))
}

pub async fn dir_pickers_items_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<Vec<dto::DirPickerItem>>> {
    Json(base.dir_pickers_items_data(id))
}

// repo_files_browsers

pub async fn repo_files_browsers_create(
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path, options)): Json<(String, String, dto::RepoFilesBrowserOptions)>,
) -> Json<u32> {
    Json(base.repo_files_browsers_create(repo_id, encrypted_path, options))
}

pub async fn repo_files_browsers_destroy(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    base.repo_files_browsers_destroy(browser_id);
}

pub async fn repo_files_browsers_info(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoFilesBrowserInfo>> {
    Json(base.repo_files_browsers_info(browser_id))
}

pub async fn repo_files_browsers_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((browser_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_files_browsers_info_subscribe(browser_id, callbacks.cb(cb)))
}

pub async fn repo_files_browsers_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoFilesBrowserInfo>> {
    Json(base.repo_files_browsers_info_data(id))
}

pub async fn repo_files_browsers_load_files(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    base.repo_files_browsers_load_files(browser_id);
}

pub async fn repo_files_browsers_select_file(
    ExtractBase(base): ExtractBase,
    Json((browser_id, file_id, extend, range, force)): Json<(u32, String, bool, bool, bool)>,
) {
    base.repo_files_browsers_select_file(browser_id, file_id, extend, range, force);
}

pub async fn repo_files_browsers_select_all(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    base.repo_files_browsers_select_all(browser_id);
}

pub async fn repo_files_browsers_clear_selection(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    base.repo_files_browsers_clear_selection(browser_id);
}

pub async fn repo_files_browsers_sort_by(
    ExtractBase(base): ExtractBase,
    Json((browser_id, field)): Json<(u32, dto::RepoFilesSortField)>,
) {
    base.repo_files_browsers_sort_by(browser_id, field);
}

pub async fn repo_files_browsers_create_dir(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    base.repo_files_browsers_create_dir(browser_id);
}

pub async fn repo_files_browsers_create_file(
    ExtractBase(base): ExtractBase,
    Json((browser_id, name)): Json<(u32, String)>,
) -> Json<Option<String>> {
    Json(base.repo_files_browsers_create_file(browser_id, name).await)
}

pub async fn repo_files_browsers_delete_selected(
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    base.repo_files_browsers_delete_selected(browser_id);
}

pub async fn repo_files_browsers_move_selected(
    ExtractBase(base): ExtractBase,
    Json((browser_id, mode)): Json<(u32, dto::RepoFilesMoveMode)>,
) {
    base.repo_files_browsers_move_selected(browser_id, mode);
}

// repo_files_details

pub async fn repo_files_details_create(
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path, is_editing, options)): Json<(
        String,
        String,
        bool,
        dto::RepoFilesDetailsOptions,
    )>,
) -> Json<u32> {
    Json(base.repo_files_details_create(repo_id, encrypted_path, is_editing, options))
}

pub async fn repo_files_details_destroy(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_destroy(details_id);
}

pub async fn repo_files_details_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((details_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_files_details_info_subscribe(details_id, callbacks.cb(cb)))
}

pub async fn repo_files_details_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoFilesDetailsInfo>> {
    Json(base.repo_files_details_info_data(id))
}

pub async fn repo_files_details_file_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((details_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_files_details_file_subscribe(details_id, callbacks.cb(cb)))
}

pub async fn repo_files_details_file_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoFile>> {
    Json(base.repo_files_details_file_data(id))
}

pub async fn repo_files_details_content_bytes_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((details_id, cb)): Json<(u32, CallbackId)>,
) -> Json<u32> {
    Json(base.repo_files_details_content_bytes_subscribe(details_id, callbacks.cb(cb)))
}

pub async fn repo_files_details_content_bytes_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Response {
    match base.repo_files_details_content_bytes_data(id) {
        Some(bytes) => BASE64.encode(&bytes).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

pub async fn repo_files_details_load_file(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_load_file(details_id);
}

pub async fn repo_files_details_load_content(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_load_content(details_id);
}

pub async fn repo_files_details_edit(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_edit(details_id);
}

pub async fn repo_files_details_edit_cancel(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_edit_cancel(details_id);
}

pub async fn repo_files_details_set_content(
    ExtractBase(base): ExtractBase,
    Json((details_id, content_base64)): Json<(u32, String)>,
) {
    base.repo_files_details_set_content(
        details_id,
        BASE64.decode(content_base64.as_bytes()).unwrap(),
    );
}

pub async fn repo_files_details_save(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_save(details_id);
}

pub async fn repo_files_details_delete(
    ExtractBase(base): ExtractBase,
    Json((details_id,)): Json<(u32,)>,
) {
    base.repo_files_details_delete(details_id);
}

// repo_files_move

pub async fn repo_files_move_info_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.repo_files_move_info_subscribe(callbacks.cb(cb)))
}

pub async fn repo_files_move_info_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::RepoFilesMoveInfo>> {
    Json(base.repo_files_move_info_data(id))
}

pub async fn repo_files_move_dir_picker_click(
    ExtractBase(base): ExtractBase,
    Json((item_id, is_arrow)): Json<(String, bool)>,
) {
    base.repo_files_move_dir_picker_click(item_id, is_arrow);
}

pub async fn repo_files_move_move_files(ExtractBase(base): ExtractBase) {
    base.repo_files_move_move_files();
}

pub async fn repo_files_move_cancel(ExtractBase(base): ExtractBase) {
    base.repo_files_move_cancel();
}

pub async fn repo_files_move_create_dir(ExtractBase(base): ExtractBase) {
    base.repo_files_move_create_dir();
}

// space_usage

pub async fn space_usage_subscribe(
    ExtractBase(base): ExtractBase,
    ExtractCallbacks(callbacks): ExtractCallbacks,
    Json((cb,)): Json<(CallbackId,)>,
) -> Json<u32> {
    Json(base.space_usage_subscribe(callbacks.cb(cb)))
}

pub async fn space_usage_data(
    ExtractBase(base): ExtractBase,
    Json((id,)): Json<(u32,)>,
) -> Json<Option<dto::SpaceUsage>> {
    Json(base.space_usage_data(id))
}

// browser integration

async fn transfers_download_reader_provider_pick_file(
    base: Arc<WebVaultBase>,
    reader_provider: repo_files_read::state::RepoFileReaderProvider,
    save_file: Arc<
        Box<dyn Fn(String) -> BoxFuture<'static, Option<PathBuf>> + Send + Sync + 'static>,
    >,
) {
    transfers_download_reader_provider_downloadable(
        base,
        reader_provider,
        Box::new(PickFileDownloadable {
            pick_file: Box::new(move |name| {
                let save_file = save_file.clone();

                async move { save_file(name).await.ok_or(DownloadableError::Aborted) }.boxed()
            }),
            on_open: Some(Box::new(move |path, _content_type| {
                let _ = open::that(path);

                Ok(())
            })),
            on_done: Box::new(move |_path, _content_type| Ok(())),
            path: None,
            content_type: None,
        }),
    )
    .await;
}

async fn transfers_download_reader_provider_downloadable(
    base: Arc<WebVaultBase>,
    reader_provider: repo_files_read::state::RepoFileReaderProvider,
    downloadable: downloadable::BoxDownloadable,
) {
    let (_, create_future) = base.vault.transfers_download(reader_provider, downloadable);

    transfers_process_create_download_result_future(base, create_future).await;
}

async fn transfers_process_create_download_result_future(
    base: Arc<WebVaultBase>,
    create_future: transfers::state::CreateDownloadResultFuture,
) {
    let future = match create_future.await {
        Ok(future) => future,
        Err(err) if matches!(err, TransferError::AlreadyExists) => return,
        Err(err) => {
            base.errors.handle_error(err);
            return;
        }
    };

    // errors are displayed in transfers
    let _ = future.await;
}

pub async fn repo_files_open_file(
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path)): Json<(String, String)>,
) {
    let repo_id = RepoId(repo_id);
    let path = EncryptedPath(encrypted_path);
    let local_base_path = std::env::temp_dir().to_str().unwrap().to_owned();

    base.clone().spawn(|vault| {
        async move {
            let reader_provider = match vault.repo_files_get_file_reader(&repo_id, &path) {
                Ok(reader_provider) => reader_provider,
                Err(err) => {
                    base.errors.handle_error(err);
                    return;
                }
            };

            transfers_download_reader_provider_downloadable(
                base,
                reader_provider,
                Box::new(TempFileDownloadable {
                    base_path: local_base_path.into(),
                    on_open: None,
                    on_done: Box::new(move |path, _content_type| {
                        let _ = open::that(path);

                        Ok(())
                    }),
                    parent_path: None,
                    temp_path: None,
                    path: None,
                    content_type: None,
                }),
            )
            .await;
        }
        .boxed()
    });
}

pub async fn repo_files_download_file(
    State(state): State<AppState>,
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path)): Json<(String, String)>,
) {
    match state.file_handlers.save_file.clone() {
        Some(save_file) => {
            let repo_id = RepoId(repo_id);
            let path = EncryptedPath(encrypted_path);

            base.clone().spawn(move |vault| {
                async move {
                    let reader_provider = match vault.repo_files_get_file_reader(&repo_id, &path) {
                        Ok(reader_provider) => reader_provider,
                        Err(err) => {
                            base.errors.handle_error(err);
                            return;
                        }
                    };

                    transfers_download_reader_provider_pick_file(base, reader_provider, save_file)
                        .await;
                }
                .boxed()
            });
        }
        None => base.vault.notifications_show("Not implemented".into()),
    }
}

fn repo_files_upload_path(
    base: Arc<WebVaultBase>,
    repo_id: RepoId,
    parent_path: EncryptedPath,
    local_path: PathBuf,
) {
    let upload_base = base.clone();
    let on_error_base = base.clone();

    upload_helper::handle_path(
        local_path,
        Box::new(move |local_path, name| {
            let base = upload_base.clone();
            let repo_id = repo_id.clone();
            let parent_path = parent_path.clone();

            base.clone().spawn(move |vault| {
                async move {
                    let uploadable = FileUploadable {
                        path: local_path,
                        cleanup: None,
                    };

                    let (_, create_future) =
                        vault.transfers_upload(repo_id, parent_path, name, Box::new(uploadable));

                    base.handle_result(match create_future.await {
                        Ok(future) => future.await.map(|_| ()),
                        Err(err) => Err(err),
                    });
                }
                .boxed()
            });
        }),
        Box::new(move |path, err| {
            on_error_base.handle_error(StringUserError(format!(
                "Failed to upload {:?}: {}",
                path, err
            )));
        }),
    );
}

pub async fn repo_files_upload_file(
    State(state): State<AppState>,
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path)): Json<(String, String)>,
) {
    match state.file_handlers.pick_files.clone() {
        Some(pick_files) => {
            let repo_id = RepoId(repo_id);
            let parent_path = EncryptedPath(encrypted_path);

            base.clone().spawn(move |_| {
                async move {
                    if let Some(local_paths) = pick_files().await {
                        for local_path in local_paths {
                            repo_files_upload_path(
                                base.clone(),
                                repo_id.clone(),
                                parent_path.clone(),
                                local_path,
                            );
                        }
                    }
                }
                .boxed()
            })
        }
        None => base.vault.notifications_show("Not implemented".into()),
    }
}

pub async fn repo_files_upload_dir(
    State(state): State<AppState>,
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path)): Json<(String, String)>,
) {
    let repo_id = RepoId(repo_id);
    let parent_path = EncryptedPath(encrypted_path);

    base.clone().spawn(move |vault| {
        async move {
            match state.file_handlers.pick_dirs.as_ref() {
                Some(pick_dirs) => {
                    if let Some(local_paths) = pick_dirs().await {
                        for local_path in local_paths {
                            repo_files_upload_path(
                                base.clone(),
                                repo_id.clone(),
                                parent_path.clone(),
                                local_path,
                            );
                        }
                    }
                }
                None => vault.notifications_show("Not implemented".into()),
            }
        }
        .boxed()
    })
}

pub async fn repo_files_upload_paths(
    ExtractBase(base): ExtractBase,
    Json((repo_id, encrypted_path, local_paths)): Json<(String, String, Vec<String>)>,
) {
    let repo_id = RepoId(repo_id);
    let parent_path = EncryptedPath(encrypted_path);

    base.clone().spawn(move |_| {
        async move {
            for local_path in local_paths {
                repo_files_upload_path(
                    base.clone(),
                    repo_id.clone(),
                    parent_path.clone(),
                    PathBuf::from(local_path),
                );
            }
        }
        .boxed()
    })
}

#[derive(Deserialize)]
pub struct RepoFilesDetailsGetFileStreamQuery {
    #[serde(rename = "detailsId")]
    pub details_id: u32,
}

pub async fn repo_files_details_get_file_stream(
    ExtractBase(base): ExtractBase,
    Query(RepoFilesDetailsGetFileStreamQuery { details_id }): Query<
        RepoFilesDetailsGetFileStreamQuery,
    >,
) -> Result<Response, ApiError> {
    let reader = match base
        .vault
        .clone()
        .repo_files_details_get_file_reader(details_id)
        .await
    {
        Ok(provider) => match provider.reader().await {
            Ok(reader) => reader,
            Err(err) => {
                base.errors.handle_error(err.clone());

                return Err(err.to_string().into());
            }
        },
        Err(err) => {
            base.errors.handle_error(err.clone());

            return Err(err.to_string().into());
        }
    };

    let (_, file_reader) = base.vault.clone().transfers_download_reader(reader);

    let stream = ReaderStream::new(file_reader.reader, BLOCK_SIZE);

    let mut res = Body::from_stream(stream).into_response();
    *res.status_mut() = StatusCode::OK;
    if let SizeInfo::Exact(size) = file_reader.size {
        res.headers_mut()
            .insert(header::CONTENT_LENGTH, size.into());
    }
    if let Some(content_type) = file_reader.content_type {
        res.headers_mut()
            .insert(header::CONTENT_TYPE, content_type.try_into().unwrap());
    }

    Ok(res)
}

pub async fn repo_files_browsers_download_selected(
    State(state): State<AppState>,
    ExtractBase(base): ExtractBase,
    Json((browser_id,)): Json<(u32,)>,
) {
    match state.file_handlers.save_file.clone() {
        Some(save_file) => {
            base.clone().spawn(move |vault| {
                async move {
                    let reader_provider =
                        match vault.repo_files_browsers_get_selected_reader(browser_id) {
                            Ok(reader_provider) => reader_provider,
                            Err(err) => {
                                base.errors.handle_error(err);
                                return;
                            }
                        };

                    transfers_download_reader_provider_pick_file(base, reader_provider, save_file)
                        .await;
                }
                .boxed()
            });
        }
        None => base.vault.notifications_show("Not implemented".into()),
    }
}
