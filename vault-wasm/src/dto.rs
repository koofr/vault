use std::time::Duration;

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use vault_core::{
    common::state as common_state,
    dialogs::state as dialogs_state,
    dir_pickers::state as dir_pickers_state,
    files::{
        self, file_category,
        file_size::{size_display, speed_display_bytes_duration},
        files_filter,
    },
    notifications::state as notifications_state,
    relative_time,
    remote_files::state as remote_files_state,
    repo_config_backup::state as repo_config_backup_state,
    repo_files::state as repo_files_state,
    repo_files_browsers::state as repo_files_browsers_state,
    repo_files_details::state as repo_files_details_state,
    repo_files_move::state as repo_files_move_state,
    repo_remove::state as repo_remove_state,
    repo_space_usage::state as repo_space_usage_state,
    repo_unlock::state as repo_unlock_state,
    repos::{selectors as repos_selectors, state as repos_state},
    selection::state as selection_state,
    sort::state as sort_state,
    space_usage::state as space_usage_state,
    store,
    transfers::{selectors as transfers_selectors, state as transfers_state},
    user::state as user_state,
    user_error::UserError,
};

use crate::browser_runtime::now_ms;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(tag = "type")]
pub enum Status {
    Initial,
    Loading,
    Loaded,
    Reloading,
    Error { error: String },
}

impl<E: std::error::Error + Clone + PartialEq + UserError> From<&common_state::Status<E>> for Status {
    fn from(status: &common_state::Status<E>) -> Self {
        match status {
            common_state::Status::Initial => Self::Initial,
            common_state::Status::Loading => Self::Loading,
            common_state::Status::Loaded => Self::Loaded,
            common_state::Status::Reloading => Self::Reloading,
            common_state::Status::Error { error } => Self::Error {
                error: error.user_error(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(tag = "type")]
pub enum SizeInfo {
    Exact { size: i64 },
    Estimate { size: i64 },
    Unknown,
}

impl From<&common_state::SizeInfo> for SizeInfo {
    fn from(status: &common_state::SizeInfo) -> Self {
        match status {
            common_state::SizeInfo::Exact(size) => Self::Exact { size: *size },
            common_state::SizeInfo::Estimate(size) => Self::Estimate { size: *size },
            common_state::SizeInfo::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum SelectionSummary {
    None,
    Partial,
    All,
}

impl From<&selection_state::SelectionSummary> for SelectionSummary {
    fn from(selection: &selection_state::SelectionSummary) -> Self {
        match selection {
            selection_state::SelectionSummary::None => Self::None,
            selection_state::SelectionSummary::Partial => Self::Partial,
            selection_state::SelectionSummary::All => Self::All,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl From<sort_state::SortDirection> for SortDirection {
    fn from(direction: sort_state::SortDirection) -> Self {
        match direction {
            sort_state::SortDirection::Asc => Self::Asc,
            sort_state::SortDirection::Desc => Self::Desc,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RelativeTime {
    pub value: f64,
    pub display: String,
    #[serde(rename = "nextUpdate")]
    pub next_update: Option<f64>,
}

impl From<relative_time::RelativeTime> for RelativeTime {
    fn from(time: relative_time::RelativeTime) -> Self {
        Self {
            value: time.value as f64,
            display: time.display,
            next_update: time.next_update.map(|x| x as f64),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum FileCategory {
    Generic,
    Folder,
    Archive,
    Audio,
    Code,
    Document,
    Image,
    Pdf,
    Presentation,
    Sheet,
    Text,
    Video,
}

impl From<&file_category::FileCategory> for FileCategory {
    fn from(category: &file_category::FileCategory) -> Self {
        match category {
            file_category::FileCategory::Generic => Self::Generic,
            file_category::FileCategory::Folder => Self::Folder,
            file_category::FileCategory::Archive => Self::Archive,
            file_category::FileCategory::Audio => Self::Audio,
            file_category::FileCategory::Code => Self::Code,
            file_category::FileCategory::Document => Self::Document,
            file_category::FileCategory::Image => Self::Image,
            file_category::FileCategory::Pdf => Self::Pdf,
            file_category::FileCategory::Presentation => Self::Presentation,
            file_category::FileCategory::Sheet => Self::Sheet,
            file_category::FileCategory::Text => Self::Text,
            file_category::FileCategory::Video => Self::Video,
        }
    }
}

impl Into<file_category::FileCategory> for FileCategory {
    fn into(self) -> file_category::FileCategory {
        match self {
            Self::Generic => file_category::FileCategory::Generic,
            Self::Folder => file_category::FileCategory::Folder,
            Self::Archive => file_category::FileCategory::Archive,
            Self::Audio => file_category::FileCategory::Audio,
            Self::Code => file_category::FileCategory::Code,
            Self::Document => file_category::FileCategory::Document,
            Self::Image => file_category::FileCategory::Image,
            Self::Pdf => file_category::FileCategory::Pdf,
            Self::Presentation => file_category::FileCategory::Presentation,
            Self::Sheet => file_category::FileCategory::Sheet,
            Self::Text => file_category::FileCategory::Text,
            Self::Video => file_category::FileCategory::Video,
        }
    }
}

impl Into<vault_file_icon::FileIconCategory> for FileCategory {
    fn into(self) -> vault_file_icon::FileIconCategory {
        match self {
            Self::Generic => vault_file_icon::FileIconCategory::Generic,
            Self::Folder => vault_file_icon::FileIconCategory::Folder,
            Self::Archive => vault_file_icon::FileIconCategory::Archive,
            Self::Audio => vault_file_icon::FileIconCategory::Audio,
            Self::Code => vault_file_icon::FileIconCategory::Code,
            Self::Document => vault_file_icon::FileIconCategory::Document,
            Self::Image => vault_file_icon::FileIconCategory::Image,
            Self::Pdf => vault_file_icon::FileIconCategory::Pdf,
            Self::Presentation => vault_file_icon::FileIconCategory::Presentation,
            Self::Sheet => vault_file_icon::FileIconCategory::Sheet,
            Self::Text => vault_file_icon::FileIconCategory::Text,
            Self::Video => vault_file_icon::FileIconCategory::Video,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct FilesFilter {
    pub categories: Vec<FileCategory>,
    pub exts: Vec<String>,
}

impl Into<files_filter::FilesFilter> for FilesFilter {
    fn into(self) -> files_filter::FilesFilter {
        files_filter::FilesFilter {
            categories: self.categories.into_iter().map(|x| x.into()).collect(),
            exts: self.exts,
        }
    }
}

// file_icon

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum FileIconSize {
    Sm,
    Lg,
}

impl Into<vault_file_icon::FileIconSize> for FileIconSize {
    fn into(self) -> vault_file_icon::FileIconSize {
        match self {
            Self::Sm => vault_file_icon::FileIconSize::Sm,
            Self::Lg => vault_file_icon::FileIconSize::Lg,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct FileIconAttrs {
    pub category: FileCategory,
    #[serde(rename = "isDl")]
    pub is_dl: bool,
    #[serde(rename = "isUl")]
    pub is_ul: bool,
    #[serde(rename = "isExport")]
    pub is_export: bool,
    #[serde(rename = "isImport")]
    pub is_import: bool,
    #[serde(rename = "isAndroid")]
    pub is_android: bool,
    #[serde(rename = "isIos")]
    pub is_ios: bool,
    #[serde(rename = "isVaultRepo")]
    pub is_vault_repo: bool,
    #[serde(rename = "isError")]
    pub is_error: bool,
}

impl From<files::file_icon::FileIconAttrs> for FileIconAttrs {
    fn from(attrs: files::file_icon::FileIconAttrs) -> Self {
        Self {
            category: (&attrs.category).into(),
            is_dl: attrs.is_dl,
            is_ul: attrs.is_ul,
            is_export: attrs.is_export,
            is_import: attrs.is_import,
            is_android: attrs.is_android,
            is_ios: attrs.is_ios,
            is_vault_repo: attrs.is_vault_repo,
            is_error: attrs.is_error,
        }
    }
}

impl Into<vault_file_icon::FileIconAttrs> for FileIconAttrs {
    fn into(self) -> vault_file_icon::FileIconAttrs {
        vault_file_icon::FileIconAttrs {
            category: self.category.into(),
            is_dl: self.is_dl,
            is_ul: self.is_ul,
            is_export: self.is_export,
            is_import: self.is_import,
            is_android: self.is_android,
            is_ios: self.is_ios,
            is_vault_repo: self.is_vault_repo,
            is_error: self.is_error,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct FileIconProps {
    pub size: FileIconSize,
    pub attrs: FileIconAttrs,
}

impl Into<vault_file_icon::FileIconProps> for FileIconProps {
    fn into(self) -> vault_file_icon::FileIconProps {
        vault_file_icon::FileIconProps {
            size: self.size.into(),
            attrs: self.attrs.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct Notification {
    pub id: u32,
    pub message: String,
}

impl From<&notifications_state::Notification> for Notification {
    fn from(notification: &notifications_state::Notification) -> Self {
        Self {
            id: notification.id,
            message: notification.message.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum DialogType {
    Alert,
    Confirm,
    Prompt,
}

impl From<&dialogs_state::DialogType> for DialogType {
    fn from(typ: &dialogs_state::DialogType) -> Self {
        match typ {
            dialogs_state::DialogType::Alert => Self::Alert,
            dialogs_state::DialogType::Confirm => Self::Confirm,
            dialogs_state::DialogType::Prompt => Self::Prompt,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum DialogButtonStyle {
    Primary,
    Destructive,
}

impl From<&dialogs_state::DialogButtonStyle> for DialogButtonStyle {
    fn from(typ: &dialogs_state::DialogButtonStyle) -> Self {
        match typ {
            dialogs_state::DialogButtonStyle::Primary => Self::Primary,
            dialogs_state::DialogButtonStyle::Destructive => Self::Destructive,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct Dialog {
    pub id: u32,
    pub status: Status,
    pub typ: DialogType,
    pub title: String,
    pub message: Option<String>,
    #[serde(rename = "inputValue")]
    pub input_value: String,
    #[serde(rename = "isInputValueValid")]
    pub is_input_value_valid: bool,
    #[serde(rename = "inputValueSelected")]
    pub input_value_selected: Option<String>,
    #[serde(rename = "inputPlaceholder")]
    pub input_placeholder: Option<String>,
    #[serde(rename = "confirmButtonText")]
    pub confirm_button_text: String,
    #[serde(rename = "confirmButtonEnabled")]
    pub confirm_button_enabled: bool,
    #[serde(rename = "confirmButtonStyle")]
    pub confirm_button_style: DialogButtonStyle,
    #[serde(rename = "cancelButtonText")]
    pub cancel_button_text: Option<String>,
}

impl<'a> From<dialogs_state::DialogInfo<'a>> for Dialog {
    fn from(dialog: dialogs_state::DialogInfo<'a>) -> Self {
        Self {
            id: dialog.id,
            status: dialog.status.into(),
            typ: dialog.typ.into(),
            title: dialog.title.clone(),
            message: dialog.message.cloned(),
            input_value: dialog.input_value.clone(),
            is_input_value_valid: dialog.is_input_value_valid,
            input_value_selected: dialog.input_value_selected.cloned(),
            input_placeholder: dialog.input_placeholder.cloned(),
            confirm_button_text: dialog.confirm_button_text.clone(),
            confirm_button_enabled: dialog.confirm_button_enabled,
            confirm_button_style: dialog.confirm_button_style.into(),
            cancel_button_text: dialog.cancel_button_text.cloned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct User {
    pub id: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub email: String,
}

impl From<&user_state::User> for User {
    fn from(user: &user_state::User) -> Self {
        Self {
            id: user.id.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            full_name: user.full_name.clone(),
            email: user.email.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RepoState {
    Locked,
    Unlocked,
}

impl From<&repos_state::RepoState> for RepoState {
    fn from(repo_state: &repos_state::RepoState) -> Self {
        match repo_state {
            repos_state::RepoState::Locked => Self::Locked,
            repos_state::RepoState::Unlocked { .. } => Self::Unlocked,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct Repo {
    pub id: String,
    pub name: String,
    #[serde(rename = "mountId")]
    pub mount_id: String,
    pub path: String,
    pub state: RepoState,
    pub added: f64,
    #[serde(rename = "webUrl")]
    pub web_url: String,
}

impl From<&repos_state::Repo> for Repo {
    fn from(repo: &repos_state::Repo) -> Self {
        Self {
            id: repo.id.clone(),
            name: repo.name.clone(),
            mount_id: repo.mount_id.clone(),
            path: repo.path.clone(),
            state: (&repo.state).into(),
            added: repo.added as f64,
            web_url: repo.web_url.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct Repos {
    pub status: Status,
    pub repos: Vec<Repo>,
}

impl From<&store::State> for Repos {
    fn from(state: &store::State) -> Self {
        Self {
            status: (&state.repos.status).into(),
            repos: repos_selectors::select_repos(state)
                .into_iter()
                .map(Repo::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoInfo {
    pub status: Status,
    pub repo: Option<Repo>,
}

impl<'a> From<&repos_state::RepoInfo<'a>> for RepoInfo {
    fn from(info: &repos_state::RepoInfo<'a>) -> Self {
        Self {
            status: (&info.status).into(),
            repo: info.repo.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RemoteFilesLocation {
    #[serde(rename = "mountId")]
    pub mount_id: String,
    pub path: String,
}

impl From<&remote_files_state::RemoteFilesLocation> for RemoteFilesLocation {
    fn from(location: &remote_files_state::RemoteFilesLocation) -> Self {
        Self {
            mount_id: location.mount_id.clone(),
            path: location.path.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoConfig {
    pub name: String,
    pub location: RemoteFilesLocation,
    pub password: String,
    pub salt: Option<String>,
    #[serde(rename = "rcloneConfig")]
    pub rclone_config: String,
}

impl From<&repos_state::RepoConfig> for RepoConfig {
    fn from(config: &repos_state::RepoConfig) -> Self {
        Self {
            name: config.name.clone(),
            location: (&config.location).into(),
            password: config.password.clone(),
            salt: config.salt.clone(),
            rclone_config: config.rclone_config.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RemoteFilesBreadcrumb {
    pub id: String,
    #[serde(rename = "mountId")]
    pub mount_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

impl From<&remote_files_state::RemoteFilesBreadcrumb> for RemoteFilesBreadcrumb {
    fn from(breadcrumb: &remote_files_state::RemoteFilesBreadcrumb) -> Self {
        Self {
            id: breadcrumb.id.clone(),
            mount_id: breadcrumb.mount_id.clone(),
            path: breadcrumb.path.clone(),
            name: breadcrumb.name.clone(),
            last: breadcrumb.last,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoCreateForm {
    #[serde(rename = "createLoadStatus")]
    pub create_load_status: Status,
    pub location: Option<RemoteFilesLocation>,
    #[serde(rename = "locationBreadcrumbs")]
    pub location_breadcrumbs: Vec<RemoteFilesBreadcrumb>,
    #[serde(rename = "locationDirPickerId")]
    pub location_dir_picker_id: Option<u32>,
    #[serde(rename = "locationDirPickerCanSelect")]
    pub location_dir_picker_can_select: bool,
    #[serde(rename = "locationDirPickerCreateDirEnabled")]
    pub location_dir_picker_create_dir_enabled: bool,
    pub password: String,
    pub salt: Option<String>,
    #[serde(rename = "fillFromRcloneConfigError")]
    pub fill_from_rclone_config_error: Option<String>,
    #[serde(rename = "canCreate")]
    pub can_create: bool,
    #[serde(rename = "createRepoStatus")]
    pub create_repo_status: Status,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoCreated {
    #[serde(rename = "repoId")]
    pub repo_id: String,
    pub config: RepoConfig,
}

impl From<&repos_state::RepoCreated> for RepoCreated {
    fn from(created: &repos_state::RepoCreated) -> Self {
        Self {
            repo_id: created.repo_id.clone(),
            config: (&created.config).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(tag = "type")]
pub enum RepoCreateInfo {
    Form(RepoCreateForm),
    Created(RepoCreated),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RepoUnlockMode {
    Unlock,
    Verify,
}

impl Into<repos_state::RepoUnlockMode> for RepoUnlockMode {
    fn into(self) -> repos_state::RepoUnlockMode {
        match self {
            Self::Unlock => repos_state::RepoUnlockMode::Unlock,
            Self::Verify => repos_state::RepoUnlockMode::Verify,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoUnlockOptions {
    pub mode: RepoUnlockMode,
}

impl Into<repo_unlock_state::RepoUnlockOptions> for RepoUnlockOptions {
    fn into(self) -> repo_unlock_state::RepoUnlockOptions {
        repo_unlock_state::RepoUnlockOptions {
            mode: self.mode.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoUnlockInfo {
    pub status: Status,
    #[serde(rename = "repoName")]
    pub repo_name: Option<String>,
}

impl<'a> From<&repo_unlock_state::RepoUnlockInfo<'a>> for RepoUnlockInfo {
    fn from(info: &repo_unlock_state::RepoUnlockInfo<'a>) -> Self {
        Self {
            status: info.status.into(),
            repo_name: info.repo_name.map(str::to_string),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoRemoveInfo {
    pub status: Status,
    #[serde(rename = "repoName")]
    pub repo_name: Option<String>,
}

impl<'a> From<&repo_remove_state::RepoRemoveInfo<'a>> for RepoRemoveInfo {
    fn from(info: &repo_remove_state::RepoRemoveInfo<'a>) -> Self {
        Self {
            status: info.status.into(),
            repo_name: info.repo_name.map(str::to_string),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoConfigBackupInfo {
    #[serde(rename = "unlockInfo")]
    pub unlock_info: RepoUnlockInfo,
    pub config: Option<RepoConfig>,
}

impl<'a> From<&repo_config_backup_state::RepoConfigBackupInfo<'a>> for RepoConfigBackupInfo {
    fn from(info: &repo_config_backup_state::RepoConfigBackupInfo<'a>) -> Self {
        Self {
            unlock_info: (&info.unlock_info).into(),
            config: info.config.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoSpaceUsageInfo {
    pub status: Status,
    #[serde(rename = "spaceUsedDisplay")]
    pub space_used_display: Option<String>,
}

impl<'a> From<&repo_space_usage_state::RepoSpaceUsageInfo<'a>> for RepoSpaceUsageInfo {
    fn from(info: &repo_space_usage_state::RepoSpaceUsageInfo<'a>) -> Self {
        Self {
            status: info.status.into(),
            space_used_display: info.space_used.map(size_display),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RemoteFileType {
    Dir,
    File,
}

impl From<&remote_files_state::RemoteFileType> for RemoteFileType {
    fn from(typ: &remote_files_state::RemoteFileType) -> Self {
        match typ {
            remote_files_state::RemoteFileType::Dir => Self::Dir,
            remote_files_state::RemoteFileType::File => Self::File,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RemoteFile {
    pub id: String,
    #[serde(rename = "mountId")]
    pub mount_id: String,
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: RemoteFileType,
    pub size: Option<f64>,
    pub modified: Option<f64>,
}

impl From<&remote_files_state::RemoteFile> for RemoteFile {
    fn from(file: &remote_files_state::RemoteFile) -> Self {
        Self {
            id: file.id.to_owned(),
            mount_id: file.mount_id.to_owned(),
            path: file.path.to_owned(),
            name: file.name.to_owned(),
            typ: (&file.typ).into(),
            size: file.size.map(|size| size as f64),
            modified: file.modified.map(|modified| modified as f64),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RepoFileType {
    Dir,
    File,
}

impl From<&repo_files_state::RepoFileType> for RepoFileType {
    fn from(typ: &repo_files_state::RepoFileType) -> Self {
        match typ {
            repo_files_state::RepoFileType::Dir => Self::Dir,
            repo_files_state::RepoFileType::File => Self::File,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFile {
    pub id: String,
    #[serde(rename = "repoId")]
    pub repo_id: String,
    pub path: Option<String>,
    pub name: String,
    #[serde(rename = "nameError")]
    pub name_error: bool,
    pub ext: Option<String>,
    #[serde(rename = "contentType")]
    pub content_type: Option<String>,
    #[serde(rename = "type")]
    pub typ: RepoFileType,
    #[serde(rename = "sizeDisplay")]
    pub size_display: String,
    pub modified: Option<f64>,
    #[serde(rename = "remoteHash")]
    pub remote_hash: Option<String>,
    pub category: FileCategory,
    #[serde(rename = "fileIconAttrs")]
    pub file_icon_attrs: FileIconAttrs,
}

impl From<&repo_files_state::RepoFile> for RepoFile {
    fn from(file: &repo_files_state::RepoFile) -> Self {
        Self {
            id: file.id.clone(),
            repo_id: file.repo_id.clone(),
            path: match &file.path {
                repo_files_state::RepoFilePath::Decrypted { path } => Some(path.clone()),
                repo_files_state::RepoFilePath::DecryptError {
                    parent_path: _,
                    encrypted_name: _,
                    error: _,
                } => None,
            },
            name: match &file.name {
                repo_files_state::RepoFileName::Decrypted { name, .. } => name.clone(),
                repo_files_state::RepoFileName::DecryptError { encrypted_name, .. } => {
                    encrypted_name.clone()
                }
            },
            name_error: match &file.name {
                repo_files_state::RepoFileName::Decrypted { .. } => false,
                repo_files_state::RepoFileName::DecryptError { .. } => true,
            },
            ext: file.ext.clone(),
            content_type: file.content_type.clone(),
            typ: (&file.typ).into(),
            size_display: match &file.size {
                Some(repo_files_state::RepoFileSize::Decrypted { size }) => size_display(*size),
                Some(repo_files_state::RepoFileSize::DecryptError {
                    encrypted_size: _,
                    error: _,
                }) => String::from("???"),
                None => "".into(),
            },
            modified: file.modified.map(|modified| modified as f64),
            remote_hash: file.remote_hash.clone(),
            category: (&file.category).into(),
            file_icon_attrs: file.file_icon_attrs().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesSort {
    field: RepoFilesSortField,
    direction: SortDirection,
}

impl From<&repo_files_state::RepoFilesSort> for RepoFilesSort {
    fn from(sort: &repo_files_state::RepoFilesSort) -> Self {
        Self {
            field: sort.field.clone().into(),
            direction: sort.direction.clone().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RepoFilesSortField {
    Name,
    Size,
    Modified,
}

impl From<repo_files_state::RepoFilesSortField> for RepoFilesSortField {
    fn from(field: repo_files_state::RepoFilesSortField) -> Self {
        match field {
            repo_files_state::RepoFilesSortField::Name => Self::Name,
            repo_files_state::RepoFilesSortField::Size => Self::Size,
            repo_files_state::RepoFilesSortField::Modified => Self::Modified,
        }
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash)]
pub enum RepoFilesSortFieldArg {
    Name,
    Size,
    Modified,
}

impl Into<repo_files_state::RepoFilesSortField> for RepoFilesSortFieldArg {
    fn into(self) -> repo_files_state::RepoFilesSortField {
        match self {
            Self::Name => repo_files_state::RepoFilesSortField::Name,
            Self::Size => repo_files_state::RepoFilesSortField::Size,
            Self::Modified => repo_files_state::RepoFilesSortField::Modified,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesBreadcrumb {
    pub id: String,
    #[serde(rename = "repoId")]
    pub repo_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

impl From<&repo_files_state::RepoFilesBreadcrumb> for RepoFilesBreadcrumb {
    fn from(breadcrumb: &repo_files_state::RepoFilesBreadcrumb) -> Self {
        Self {
            id: breadcrumb.id.clone(),
            repo_id: breadcrumb.repo_id.clone(),
            path: breadcrumb.path.clone(),
            name: breadcrumb.name.clone(),
            last: breadcrumb.last,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesUploadResult {
    pub file_id: String,
    pub name: String,
}

impl From<repo_files_state::RepoFilesUploadResult> for RepoFilesUploadResult {
    fn from(result: repo_files_state::RepoFilesUploadResult) -> Self {
        Self {
            file_id: result.file_id,
            name: result.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesBrowserOptions {
    #[serde(rename = "selectName")]
    pub select_name: Option<String>,
}

impl Into<repo_files_browsers_state::RepoFilesBrowserOptions> for RepoFilesBrowserOptions {
    fn into(self) -> repo_files_browsers_state::RepoFilesBrowserOptions {
        repo_files_browsers_state::RepoFilesBrowserOptions {
            select_name: self.select_name,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesBrowserItem {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "isSelected")]
    pub is_selected: bool,
}

impl<'a> From<&repo_files_browsers_state::RepoFilesBrowserItem<'a>> for RepoFilesBrowserItem {
    fn from(item: &repo_files_browsers_state::RepoFilesBrowserItem<'a>) -> Self {
        Self {
            file_id: item.file.id.clone(),
            is_selected: item.is_selected,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesBrowserInfo {
    #[serde(rename = "repoId")]
    pub repo_id: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "selectionSummary")]
    pub selection_summary: SelectionSummary,
    pub sort: RepoFilesSort,
    pub status: Status,
    #[serde(rename = "totalCount")]
    pub total_count: usize,
    #[serde(rename = "totalSizeDisplay")]
    pub total_size_display: String,
    #[serde(rename = "selectedCount")]
    pub selected_count: usize,
    #[serde(rename = "selectedSizeDisplay")]
    pub selected_size_display: String,
    #[serde(rename = "selectedFile")]
    pub selected_file: Option<RepoFile>,
    #[serde(rename = "canDownloadSelected")]
    pub can_download_selected: bool,
    #[serde(rename = "canCopySelected")]
    pub can_copy_selected: bool,
    #[serde(rename = "canMoveSelected")]
    pub can_move_selected: bool,
    #[serde(rename = "canDeleteSelected")]
    pub can_delete_selected: bool,
}

impl<'a> From<&repo_files_browsers_state::RepoFilesBrowserInfo<'a>> for RepoFilesBrowserInfo {
    fn from(info: &repo_files_browsers_state::RepoFilesBrowserInfo<'a>) -> Self {
        Self {
            repo_id: info.repo_id.map(str::to_string),
            path: info.path.map(str::to_string),
            selection_summary: (&info.selection_summary).into(),
            sort: (&info.sort).into(),
            status: info.status.into(),
            total_count: info.total_count,
            total_size_display: size_display(info.total_size),
            selected_count: info.selected_count,
            selected_size_display: size_display(info.selected_size),
            selected_file: info.selected_file.map(Into::into),
            can_download_selected: info.can_download_selected,
            can_copy_selected: info.can_copy_selected,
            can_move_selected: info.can_move_selected,
            can_delete_selected: info.can_delete_selected,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesDetailsOptions {
    #[serde(rename = "loadContent")]
    pub load_content: FilesFilter,
    #[serde(rename = "autosaveIntervalMs")]
    pub autosave_interval_ms: u32,
}

impl Into<repo_files_details_state::RepoFilesDetailsOptions> for RepoFilesDetailsOptions {
    fn into(self) -> repo_files_details_state::RepoFilesDetailsOptions {
        repo_files_details_state::RepoFilesDetailsOptions {
            load_content: self.load_content.into(),
            autosave_interval: Duration::from_millis(self.autosave_interval_ms as u64),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesDetailsInfo {
    #[serde(rename = "repoId")]
    pub repo_id: Option<String>,
    #[serde(rename = "parentPath")]
    pub parent_path: Option<String>,
    pub path: Option<String>,
    pub status: Status,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "fileExt")]
    pub file_ext: Option<String>,
    #[serde(rename = "fileCategory")]
    pub file_category: Option<FileCategory>,
    #[serde(rename = "fileModified")]
    pub file_modified: Option<f64>,
    #[serde(rename = "fileExists")]
    pub file_exists: bool,
    #[serde(rename = "contentStatus")]
    pub content_status: Status,
    #[serde(rename = "saveStatus")]
    pub save_status: Status,
    pub error: Option<String>,
    #[serde(rename = "isEditing")]
    pub is_editing: bool,
    #[serde(rename = "isDirty")]
    pub is_dirty: bool,
    #[serde(rename = "shouldDestroy")]
    pub should_destroy: bool,
    #[serde(rename = "canSave")]
    pub can_save: bool,
    #[serde(rename = "canDownload")]
    pub can_download: bool,
    #[serde(rename = "canCopy")]
    pub can_copy: bool,
    #[serde(rename = "canMove")]
    pub can_move: bool,
    #[serde(rename = "canDelete")]
    pub can_delete: bool,
}

impl<'a> From<&repo_files_details_state::RepoFilesDetailsInfo<'a>> for RepoFilesDetailsInfo {
    fn from(info: &repo_files_details_state::RepoFilesDetailsInfo<'a>) -> Self {
        Self {
            repo_id: info.repo_id.map(str::to_string),
            parent_path: info.parent_path.map(str::to_string),
            path: info.path.map(str::to_string),
            status: (&info.status).into(),
            file_name: info.file_name.map(str::to_string),
            file_ext: info.file_ext.clone(),
            file_category: info.file_category.as_ref().map(Into::into),
            file_modified: info.file_modified.map(|x| x as f64),
            file_exists: info.file_exists,
            content_status: (&info.content_status).into(),
            save_status: (&info.save_status).into(),
            error: info.error.clone(),
            is_editing: info.is_editing,
            is_dirty: info.is_dirty,
            should_destroy: info.should_destroy,
            can_save: info.can_save,
            can_download: info.can_download,
            can_copy: info.can_copy,
            can_move: info.can_move,
            can_delete: info.can_delete,
        }
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Eq, Hash)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

impl Into<repo_files_move_state::RepoFilesMoveMode> for RepoFilesMoveMode {
    fn into(self) -> repo_files_move_state::RepoFilesMoveMode {
        match self {
            Self::Copy => repo_files_move_state::RepoFilesMoveMode::Copy,
            Self::Move => repo_files_move_state::RepoFilesMoveMode::Move,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RepoFilesMoveInfoMode {
    Copy,
    Move,
}

impl From<&repo_files_move_state::RepoFilesMoveMode> for RepoFilesMoveInfoMode {
    fn from(typ: &repo_files_move_state::RepoFilesMoveMode) -> Self {
        match typ {
            repo_files_move_state::RepoFilesMoveMode::Copy => Self::Copy,
            repo_files_move_state::RepoFilesMoveMode::Move => Self::Move,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesMoveInfo {
    #[serde(rename = "srcFilesCount")]
    pub src_files_count: usize,
    pub mode: RepoFilesMoveInfoMode,
    #[serde(rename = "dirPickerId")]
    pub dir_picker_id: u32,
    #[serde(rename = "destFileName")]
    pub dest_file_name: Option<String>,
    #[serde(rename = "createDirEnabled")]
    pub create_dir_enabled: bool,
    #[serde(rename = "canMove")]
    pub can_move: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum TransferType {
    Upload,
    Download,
}

impl From<&transfers_state::TransferType> for TransferType {
    fn from(typ: &transfers_state::TransferType) -> Self {
        match typ {
            transfers_state::TransferType::Upload(..) => Self::Upload,
            transfers_state::TransferType::Download(..) => Self::Download,
            transfers_state::TransferType::DownloadReader => Self::Download,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(tag = "type")]
pub enum TransferState {
    Waiting,
    Processing,
    Transferring,
    Failed { error: String },
    Done,
}

impl From<&transfers_state::TransferState> for TransferState {
    fn from(typ: &transfers_state::TransferState) -> Self {
        match typ {
            transfers_state::TransferState::Waiting => Self::Waiting,
            transfers_state::TransferState::Processing => Self::Processing,
            transfers_state::TransferState::Transferring => Self::Transferring,
            transfers_state::TransferState::Failed { error } => Self::Failed {
                error: error.user_error(),
            },
            transfers_state::TransferState::Done => Self::Done,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct Transfer {
    pub id: u32,
    pub typ: TransferType,
    pub name: String,
    #[serde(rename = "fileIconAttrs")]
    pub file_icon_attrs: FileIconAttrs,
    pub size: Option<i64>,
    #[serde(rename = "sizeDisplay")]
    pub size_display: Option<String>,
    #[serde(rename = "transferredBytes")]
    pub transferred_bytes: i64,
    #[serde(rename = "transferredDisplay")]
    pub transferred_display: String,
    #[serde(rename = "speedDisplay")]
    pub speed_display: Option<String>,
    pub state: TransferState,
    #[serde(rename = "canRetry")]
    pub can_retry: bool,
    #[serde(rename = "canAbort")]
    pub can_abort: bool,
}

impl From<&transfers_state::Transfer> for Transfer {
    fn from(transfer: &transfers_state::Transfer) -> Self {
        Self {
            id: transfer.id,
            typ: (&transfer.typ).into(),
            name: transfer.name.clone(),
            file_icon_attrs: transfer.file_icon_attrs().into(),
            size: transfer.size.exact_or_estimate(),
            size_display: transfer.size.exact_or_estimate().map(size_display),
            transferred_bytes: transfer.transferred_bytes,
            transferred_display: size_display(transfer.transferred_bytes),
            speed_display: transfers_selectors::transfer_duration(&transfer, now_ms())
                .map(|duration| speed_display_bytes_duration(transfer.transferred_bytes, duration)),
            state: (&transfer.state).into(),
            can_retry: transfers_selectors::can_retry(transfer),
            can_abort: transfers_selectors::can_abort(transfer),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct TransfersList {
    pub transfers: Vec<Transfer>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct TransfersSummary {
    #[serde(rename = "totalCount")]
    pub total_count: usize,
    #[serde(rename = "doneCount")]
    pub done_count: usize,
    #[serde(rename = "failedCount")]
    pub failed_count: usize,
    #[serde(rename = "sizeProgressDisplay")]
    pub size_progress_display: String,
    pub percentage: u8,
    #[serde(rename = "remainingTimeDisplay")]
    pub remaining_time_display: String,
    #[serde(rename = "speedDisplay")]
    pub speed_display: String,
    #[serde(rename = "isTransferring")]
    pub is_transferring: bool,
    #[serde(rename = "canRetryAll")]
    pub can_retry_all: bool,
    #[serde(rename = "canAbortAll")]
    pub can_abort_all: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum DirPickerItemType {
    Folder,
    Import,
    Export,
    Hosted,
    Desktop,
    DesktopOffline,
    Dropbox,
    Googledrive,
    Onedrive,
    Bookmarks,
    Bookmark,
    Shared,
    Repo,
}

impl From<&dir_pickers_state::DirPickerItemType> for DirPickerItemType {
    fn from(typ: &dir_pickers_state::DirPickerItemType) -> Self {
        match typ {
            dir_pickers_state::DirPickerItemType::Folder => DirPickerItemType::Folder,
            dir_pickers_state::DirPickerItemType::Import => DirPickerItemType::Import,
            dir_pickers_state::DirPickerItemType::Export => DirPickerItemType::Export,
            dir_pickers_state::DirPickerItemType::Hosted => DirPickerItemType::Hosted,
            dir_pickers_state::DirPickerItemType::Desktop => DirPickerItemType::Desktop,
            dir_pickers_state::DirPickerItemType::DesktopOffline => {
                DirPickerItemType::DesktopOffline
            }
            dir_pickers_state::DirPickerItemType::Dropbox => DirPickerItemType::Dropbox,
            dir_pickers_state::DirPickerItemType::Googledrive => DirPickerItemType::Googledrive,
            dir_pickers_state::DirPickerItemType::Onedrive => DirPickerItemType::Onedrive,
            dir_pickers_state::DirPickerItemType::Bookmarks => DirPickerItemType::Bookmarks,
            dir_pickers_state::DirPickerItemType::Bookmark => DirPickerItemType::Bookmark,
            dir_pickers_state::DirPickerItemType::Shared => DirPickerItemType::Shared,
            dir_pickers_state::DirPickerItemType::Repo => DirPickerItemType::Repo,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct DirPickerItem {
    pub id: String,
    #[serde(rename = "fileId")]
    pub file_id: Option<String>,
    pub typ: DirPickerItemType,
    #[serde(rename = "isOpen")]
    pub is_open: bool,
    #[serde(rename = "isSelected")]
    pub is_selected: bool,
    #[serde(rename = "isLoading")]
    pub is_loading: bool,
    pub spaces: u16,
    #[serde(rename = "hasArrow")]
    pub has_arrow: bool,
    pub text: String,
}

impl From<&dir_pickers_state::DirPickerItem> for DirPickerItem {
    fn from(item: &dir_pickers_state::DirPickerItem) -> Self {
        Self {
            id: item.id.clone(),
            file_id: item.file_id.clone(),
            typ: (&item.typ).into(),
            is_open: item.is_open,
            is_selected: item.is_selected,
            is_loading: item.is_loading,
            spaces: item.spaces,
            has_arrow: item.has_arrow,
            text: item.text.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum SpaceUsageSeverity {
    Normal,
    Warn,
    Critical,
}

impl From<&space_usage_state::SpaceUsageSeverity> for SpaceUsageSeverity {
    fn from(severity: &space_usage_state::SpaceUsageSeverity) -> Self {
        match severity {
            space_usage_state::SpaceUsageSeverity::Normal => Self::Normal,
            space_usage_state::SpaceUsageSeverity::Warn => Self::Warn,
            space_usage_state::SpaceUsageSeverity::Critical => Self::Critical,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct SpaceUsage {
    #[serde(rename = "usedDisplay")]
    pub used_display: String,
    #[serde(rename = "totalDisplay")]
    pub total_display: String,
    pub percentage: u8,
    pub severity: SpaceUsageSeverity,
}

impl From<&space_usage_state::SpaceUsage> for SpaceUsage {
    fn from(space_usage: &space_usage_state::SpaceUsage) -> Self {
        Self {
            used_display: size_display(space_usage.used),
            total_display: size_display(space_usage.total),
            percentage: space_usage.percentage,
            severity: (&space_usage.severity).into(),
        }
    }
}
