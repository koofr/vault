use instant::Duration;
use serde::{Deserialize, Serialize};
use size;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use vault_core::{
    common::state as common_state,
    dialogs::state as dialogs_state,
    dir_pickers::state as dir_pickers_state,
    file_types::{file_category, files_filter},
    notifications::state as notifications_state,
    remote_files::state as remote_files_state,
    repo_config_backup::state as repo_config_backup_state,
    repo_create::state as repo_create_state,
    repo_files::state as repo_files_state,
    repo_files_browsers::state as repo_files_browsers_state,
    repo_files_details::state as repo_files_details_state,
    repo_files_move::state as repo_files_move_state,
    repo_remove::state as repo_remove_state,
    repo_space_usage::state as repo_space_usage_state,
    repos::{selectors as repos_selectors, state as repos_state},
    selection,
    space_usage::state as space_usage_state,
    store,
    uploads::state as uploads_state,
    user::state as user_state,
    user_error::UserError,
};

pub fn format_size(bytes: i64) -> String {
    size::Size::from_bytes(bytes)
        .format()
        .with_style(size::Style::Abbreviated)
        .with_base(size::Base::Base2)
        .to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[serde(tag = "type")]
pub enum Status {
    Initial,
    Loading,
    Loaded,
    Reloading,
    Error { error: String },
}

impl<E: UserError + Clone> From<&common_state::Status<E>> for Status {
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
pub struct RemainingTime {
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl From<&common_state::RemainingTime> for RemainingTime {
    fn from(remaining_time: &common_state::RemainingTime) -> Self {
        Self {
            days: remaining_time.days,
            hours: remaining_time.hours,
            minutes: remaining_time.minutes,
            seconds: remaining_time.seconds,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum SelectionSummary {
    None,
    Partial,
    All,
}

impl From<&selection::state::SelectionSummary> for SelectionSummary {
    fn from(selection: &selection::state::SelectionSummary) -> Self {
        match selection {
            selection::state::SelectionSummary::None => Self::None,
            selection::state::SelectionSummary::Partial => Self::Partial,
            selection::state::SelectionSummary::All => Self::All,
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
    #[serde(rename = "initStatus")]
    pub init_status: Status,
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
    #[serde(rename = "createStatus")]
    pub create_status: Status,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoCreated {
    #[serde(rename = "repoId")]
    pub repo_id: String,
    pub config: RepoConfig,
}

impl From<&repo_create_state::RepoCreated> for RepoCreated {
    fn from(created: &repo_create_state::RepoCreated) -> Self {
        Self {
            repo_id: created.repo_id.clone(),
            config: (&created.config).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoCreateInfo {
    pub form: Option<RepoCreateForm>,
    pub created: Option<RepoCreated>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoUnlockInfo {
    pub status: Status,
    #[serde(rename = "repoName")]
    pub repo_name: Option<String>,
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
    pub status: Status,
    pub config: Option<RepoConfig>,
}

impl<'a> From<&repo_config_backup_state::RepoConfigBackupInfo<'a>> for RepoConfigBackupInfo {
    fn from(info: &repo_config_backup_state::RepoConfigBackupInfo<'a>) -> Self {
        Self {
            status: info.status.into(),
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
            space_used_display: info.space_used.map(format_size),
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
    pub size: f64,
    pub modified: f64,
}

impl From<&remote_files_state::RemoteFile> for RemoteFile {
    fn from(file: &remote_files_state::RemoteFile) -> Self {
        Self {
            id: file.id.to_owned(),
            mount_id: file.mount_id.to_owned(),
            path: file.path.to_owned(),
            name: file.name.to_owned(),
            typ: (&file.typ).into(),
            size: file.size as f64,
            modified: file.modified as f64,
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
    pub modified: f64,
    #[serde(rename = "remoteHash")]
    pub remote_hash: Option<String>,
    pub category: FileCategory,
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
            size_display: match &file.typ {
                repo_files_state::RepoFileType::File => match file.size {
                    repo_files_state::RepoFileSize::Decrypted { size } => format_size(size),
                    repo_files_state::RepoFileSize::DecryptError {
                        encrypted_size: _,
                        error: _,
                    } => String::from("???"),
                },
                repo_files_state::RepoFileType::Dir => String::from(""),
            },
            modified: file.modified as f64,
            remote_hash: file.remote_hash.clone(),
            category: (&file.category).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct RepoFilesSort {
    field: RepoFilesSortField,
    direction: RepoFilesSortDirection,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum RepoFilesSortDirection {
    Asc,
    Desc,
}

impl From<repo_files_state::RepoFilesSortDirection> for RepoFilesSortDirection {
    fn from(direction: repo_files_state::RepoFilesSortDirection) -> Self {
        match direction {
            repo_files_state::RepoFilesSortDirection::Asc => Self::Asc,
            repo_files_state::RepoFilesSortDirection::Desc => Self::Desc,
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
            total_size_display: format_size(info.total_size),
            selected_count: info.selected_count,
            selected_size_display: format_size(info.selected_size),
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

pub fn format_speed(bytes: i64, duration: Duration) -> Option<String> {
    if duration.is_zero() {
        return None;
    }

    let speed = (bytes as f64 / duration.as_secs_f64()) as i64;

    Some(format!("{}/s", format_size(speed)))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub enum FileUploadState {
    Waiting,
    Uploading,
    Failed,
    Done,
}

impl From<&uploads_state::FileUploadState> for FileUploadState {
    fn from(typ: &uploads_state::FileUploadState) -> Self {
        match typ {
            uploads_state::FileUploadState::Waiting => Self::Waiting,
            uploads_state::FileUploadState::Uploading => Self::Uploading,
            uploads_state::FileUploadState::Failed { .. } => Self::Failed,
            uploads_state::FileUploadState::Done => Self::Done,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct FileUpload {
    pub id: u32,
    pub name: String,
    pub category: FileCategory,
    pub size: Option<i64>,
    #[serde(rename = "sizeDisplay")]
    pub size_display: Option<String>,
    pub uploaded: i64,
    #[serde(rename = "uploadedDisplay")]
    pub uploaded_display: String,
    pub elapsed: f64,
    pub speed: Option<String>,
    pub state: FileUploadState,
    pub error: Option<String>,
}

impl From<&uploads_state::FileUpload> for FileUpload {
    fn from(file: &uploads_state::FileUpload) -> Self {
        let elapsed = instant::now() - file.started as f64;

        Self {
            id: file.id,
            name: file.name.clone(),
            category: (&file.category).into(),
            size: file.size,
            size_display: file.size.map(format_size),
            uploaded: file.uploaded_bytes,
            uploaded_display: format_size(file.uploaded_bytes),
            elapsed,
            speed: format_speed(file.uploaded_bytes, Duration::from_millis(elapsed as u64)),
            state: (&file.state).into(),
            error: match &file.state {
                uploads_state::FileUploadState::Failed { error } => Some(error.user_error()),
                _ => None,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct UploadsFiles {
    pub files: Vec<FileUpload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
pub struct UploadsSummary {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
    #[serde(rename = "doneCount")]
    pub done_count: u32,
    #[serde(rename = "failedCount")]
    pub failed_count: u32,
    #[serde(rename = "totalBytes")]
    pub total_bytes: i64,
    #[serde(rename = "doneBytes")]
    pub done_bytes: i64,
    pub percentage: u8,
    #[serde(rename = "remainingTime")]
    pub remaining_time: RemainingTime,
    #[serde(rename = "bytesPerSecond")]
    pub bytes_per_second: f64,
    #[serde(rename = "isUploading")]
    pub is_uploading: bool,
    #[serde(rename = "canRetry")]
    pub can_retry: bool,
    #[serde(rename = "canAbort")]
    pub can_abort: bool,
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
            used_display: format_size(space_usage.used),
            total_display: format_size(space_usage.total),
            percentage: space_usage.percentage,
            severity: (&space_usage.severity).into(),
        }
    }
}
