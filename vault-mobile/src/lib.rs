pub mod download_stream_writer;
pub mod memory_secure_storage;
pub mod mobile_downloadable;
pub mod mobile_errors;
pub mod mobile_logger;
pub mod mobile_secure_storage;
pub mod mobile_spawn;
pub mod mobile_subscription;
pub mod mobile_uploadable;
pub mod upload_stream_reader;

use std::{
    collections::{hash_map, HashMap},
    fmt::Debug,
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use lazy_static::lazy_static;
use thiserror::Error;

use vault_core::{
    common::state as common_state,
    dialogs::state as dialogs_state,
    files::{self, file_category, files_filter},
    notifications::state as notifications_state,
    oauth2::OAuth2Config,
    relative_time,
    remote_files::state as remote_files_state,
    remote_files_browsers::state::{self as remote_files_browsers_state, RemoteFilesBrowserItemId},
    repo_files::state as repo_files_state,
    repo_files_browsers::state as repo_files_browsers_state,
    repo_files_details::state as repo_files_details_state,
    repo_files_move::state as repo_files_move_state,
    repo_files_read,
    repo_remove::state as repo_remove_state,
    repo_unlock::state as repo_unlock_state,
    repos::{selectors as repos_selectors, state as repos_state},
    selection::state as selection_state,
    sort::state as sort_state,
    store::{self, Event},
    transfers::{
        self, downloadable, errors::TransferError, selectors as transfers_selectors,
        state as transfers_state,
    },
    types::{DecryptedName, EncryptedPath, MountId, RemoteName, RemotePath, RepoFileId, RepoId},
    user::state as user_state,
    user_error::UserError,
    Vault,
};
use vault_native::{native_runtime::now_ms, vault::build_vault};

use crate::{
    mobile_downloadable::MobileDownloadable, mobile_errors::MobileErrors,
    mobile_secure_storage::MobileSecureStorage, mobile_spawn::MobileSpawn,
    mobile_subscription::MobileSubscription, mobile_uploadable::MobileUploadable,
};

// logging

#[derive(Debug)]
pub enum LoggerLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Into<log::Level> for LoggerLevel {
    fn into(self) -> log::Level {
        match self {
            Self::Error => log::Level::Error,
            Self::Warn => log::Level::Warn,
            Self::Info => log::Level::Info,
            Self::Debug => log::Level::Debug,
            Self::Trace => log::Level::Trace,
        }
    }
}

impl From<log::Level> for LoggerLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => Self::Error,
            log::Level::Warn => Self::Warn,
            log::Level::Info => Self::Info,
            log::Level::Debug => Self::Debug,
            log::Level::Trace => Self::Trace,
        }
    }
}

pub trait LoggerCallback: Send + Sync + Debug {
    fn log(&self, level: LoggerLevel, message: String);
}

// secure_storage

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SecureStorageError {
    #[error("secure storage error: {reason}")]
    StorageError { reason: String },
}

impl From<uniffi::UnexpectedUniFFICallbackError> for SecureStorageError {
    fn from(err: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::StorageError { reason: err.reason }
    }
}

impl From<std::io::Error> for SecureStorageError {
    fn from(err: std::io::Error) -> Self {
        Self::StorageError {
            reason: err.to_string(),
        }
    }
}

pub trait SecureStorage: Send + Sync + Debug {
    fn get_item(&self, key: String) -> Result<Option<String>, SecureStorageError>;
    fn set_item(&self, key: String, value: String) -> Result<(), SecureStorageError>;
    fn remove_item(&self, key: String) -> Result<(), SecureStorageError>;
    fn clear(&self) -> Result<(), SecureStorageError>;
}

// subscription

pub trait SubscriptionCallback: Send + Sync + Debug {
    fn on_change(&self);
}

// status

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Initial,
    Loading { loaded: bool },
    Loaded,
    // Error is a reserved keyword in uniffi UDL
    Err { error: String, loaded: bool },
}

impl<E: std::error::Error + Clone + PartialEq + UserError> From<&common_state::Status<E>>
    for Status
{
    fn from(status: &common_state::Status<E>) -> Self {
        match status {
            common_state::Status::Initial => Self::Initial,
            common_state::Status::Loading { loaded } => Self::Loading { loaded: *loaded },
            common_state::Status::Loaded => Self::Loaded,
            common_state::Status::Error { error, loaded } => Self::Err {
                error: error.user_error(),
                loaded: *loaded,
            },
        }
    }
}

// selection

#[derive(Clone, Debug, PartialEq)]
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

// sort

#[derive(Clone, Debug, PartialEq)]
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

impl Into<sort_state::SortDirection> for SortDirection {
    fn into(self) -> sort_state::SortDirection {
        match self {
            Self::Asc => sort_state::SortDirection::Asc,
            Self::Desc => sort_state::SortDirection::Desc,
        }
    }
}

// relative_time

#[derive(Clone, Debug, PartialEq)]
pub struct RelativeTime {
    pub value: i64,
    pub display: String,
    pub next_update: Option<i64>,
}

impl From<relative_time::RelativeTime> for RelativeTime {
    fn from(time: relative_time::RelativeTime) -> Self {
        Self {
            value: time.value,
            display: time.display,
            next_update: time.next_update,
        }
    }
}

// files

#[derive(Clone, Debug, PartialEq)]
pub enum SizeInfo {
    Exact { size: i64 },
    Estimate { size: i64 },
    Unknown,
}

impl Into<common_state::SizeInfo> for SizeInfo {
    fn into(self) -> common_state::SizeInfo {
        match self {
            Self::Exact { size } => common_state::SizeInfo::Exact(size),
            Self::Estimate { size } => common_state::SizeInfo::Estimate(size),
            Self::Unknown => common_state::SizeInfo::Unknown,
        }
    }
}

impl From<common_state::SizeInfo> for SizeInfo {
    fn from(size: common_state::SizeInfo) -> Self {
        match size {
            common_state::SizeInfo::Exact(size) => Self::Exact { size },
            common_state::SizeInfo::Estimate(size) => Self::Estimate { size },
            common_state::SizeInfo::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    fn from(typ: &file_category::FileCategory) -> Self {
        match typ {
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileIconAttrs {
    pub category: FileCategory,
    pub is_dl: bool,
    pub is_ul: bool,
    pub is_download_transfer: bool,
    pub is_upload_transfer: bool,
    pub is_export: bool,
    pub is_import: bool,
    pub is_android: bool,
    pub is_ios: bool,
    pub is_vault_repo: bool,
    pub is_error: bool,
}

impl From<&files::file_icon::FileIconAttrs> for FileIconAttrs {
    fn from(attrs: &files::file_icon::FileIconAttrs) -> Self {
        Self {
            category: (&attrs.category).into(),
            is_dl: attrs.is_dl,
            is_ul: attrs.is_ul,
            is_download_transfer: attrs.is_download_transfer,
            is_upload_transfer: attrs.is_upload_transfer,
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
            is_download_transfer: self.is_download_transfer,
            is_upload_transfer: self.is_upload_transfer,
            is_export: self.is_export,
            is_import: self.is_import,
            is_android: self.is_android,
            is_ios: self.is_ios,
            is_vault_repo: self.is_vault_repo,
            is_error: self.is_error,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct FileIconPng {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

// streams

#[derive(Error, Debug, Clone, PartialEq)]
pub enum StreamError {
    #[error("{reason}")]
    IoError { reason: String },
    #[error("not retriable")]
    NotRetriable,
    #[error("not openable")]
    NotOpenable,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for StreamError {
    fn from(err: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::IoError { reason: err.reason }
    }
}

impl From<std::io::Error> for StreamError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            reason: err.to_string(),
        }
    }
}

impl Into<std::io::Error> for StreamError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::BrokenPipe, self)
    }
}

impl Into<transfers::errors::UploadableError> for StreamError {
    fn into(self) -> transfers::errors::UploadableError {
        match self {
            Self::IoError { reason } => transfers::errors::UploadableError::LocalFileError(reason),
            Self::NotRetriable => transfers::errors::UploadableError::NotRetriable,
            Self::NotOpenable => {
                transfers::errors::UploadableError::LocalFileError("not openable".into())
            }
        }
    }
}

impl Into<transfers::errors::DownloadableError> for StreamError {
    fn into(self) -> transfers::errors::DownloadableError {
        match self {
            Self::IoError { reason } => {
                transfers::errors::DownloadableError::LocalFileError(reason)
            }
            Self::NotRetriable => transfers::errors::DownloadableError::NotRetriable,
            Self::NotOpenable => transfers::errors::DownloadableError::NotOpenable,
        }
    }
}

pub trait UploadStream: Send + Sync + Debug {
    fn read(&self) -> Result<Vec<u8>, StreamError>;
    fn close(&self) -> Result<(), StreamError>;
}

pub trait UploadStreamProvider: Send + Sync + Debug {
    fn size(&self) -> Result<SizeInfo, StreamError>;
    fn is_retriable(&self) -> Result<bool, StreamError>;
    fn stream(&self) -> Result<Box<dyn UploadStream>, StreamError>;
    fn dispose(&self) -> Result<(), StreamError>;
}

pub trait DownloadStream: Send + Sync + Debug {
    fn write(&self, buf: Vec<u8>) -> Result<(), StreamError>;
    fn close(&self) -> Result<(), StreamError>;
}

pub trait DownloadStreamProvider: Send + Sync + Debug {
    fn is_retriable(&self) -> Result<bool, StreamError>;
    fn is_openable(&self) -> Result<bool, StreamError>;
    fn stream(
        &self,
        name: String,
        size: SizeInfo,
        content_type: Option<String>,
        unique_name: Option<String>,
    ) -> Result<Box<dyn DownloadStream>, StreamError>;
    fn done(&self, error: Option<String>) -> Result<(), StreamError>;
    fn open(&self) -> Result<(), StreamError>;
    fn dispose(&self) -> Result<(), StreamError>;
}

// notifications

#[derive(Clone, Debug, PartialEq)]
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

// dialogs

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Dialog {
    pub id: u32,
    pub typ: DialogType,
    pub title: String,
    pub message: Option<String>,
    pub input_value: String,
    pub is_input_value_valid: bool,
    pub input_value_selected: Option<String>,
    pub input_placeholder: Option<String>,
    pub confirm_button_text: String,
    pub confirm_button_enabled: bool,
    pub confirm_button_style: DialogButtonStyle,
    pub cancel_button_text: Option<String>,
}

impl<'a> From<dialogs_state::DialogInfo<'a>> for Dialog {
    fn from(dialog: dialogs_state::DialogInfo<'a>) -> Self {
        Self {
            id: dialog.id,
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

// oauth2

pub trait OAuth2FinishFlowDone: Send + Sync + Debug {
    fn on_done(&self);
}

// user

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
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

// remote_files

#[derive(Clone, Debug, PartialEq)]
pub enum MountOrigin {
    Hosted,
    Desktop,
    Dropbox,
    Googledrive,
    Onedrive,
    Share,
    Other,
}

impl From<&remote_files_state::MountOrigin> for MountOrigin {
    fn from(origin: &remote_files_state::MountOrigin) -> Self {
        match origin {
            remote_files_state::MountOrigin::Hosted => Self::Hosted,
            remote_files_state::MountOrigin::Desktop => Self::Desktop,
            remote_files_state::MountOrigin::Dropbox => Self::Dropbox,
            remote_files_state::MountOrigin::Googledrive => Self::Googledrive,
            remote_files_state::MountOrigin::Onedrive => Self::Onedrive,
            remote_files_state::MountOrigin::Share => Self::Share,
            remote_files_state::MountOrigin::Other { .. } => Self::Other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum RemoteFilesSortField {
    Name,
    Size,
    Modified,
}

impl From<remote_files_state::RemoteFilesSortField> for RemoteFilesSortField {
    fn from(field: remote_files_state::RemoteFilesSortField) -> Self {
        match field {
            remote_files_state::RemoteFilesSortField::Name => Self::Name,
            remote_files_state::RemoteFilesSortField::Size => Self::Size,
            remote_files_state::RemoteFilesSortField::Modified => Self::Modified,
        }
    }
}

impl Into<remote_files_state::RemoteFilesSortField> for RemoteFilesSortField {
    fn into(self) -> remote_files_state::RemoteFilesSortField {
        match self {
            Self::Name => remote_files_state::RemoteFilesSortField::Name,
            Self::Size => remote_files_state::RemoteFilesSortField::Size,
            Self::Modified => remote_files_state::RemoteFilesSortField::Modified,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesSort {
    field: RemoteFilesSortField,
    direction: SortDirection,
}

impl From<&remote_files_state::RemoteFilesSort> for RemoteFilesSort {
    fn from(sort: &remote_files_state::RemoteFilesSort) -> Self {
        Self {
            field: sort.field.clone().into(),
            direction: sort.direction.clone().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesLocation {
    pub mount_id: String,
    pub path: String,
}

impl From<&remote_files_state::RemoteFilesLocation> for RemoteFilesLocation {
    fn from(location: &remote_files_state::RemoteFilesLocation) -> Self {
        Self {
            mount_id: location.mount_id.0.clone(),
            path: location.path.0.clone(),
        }
    }
}

impl Into<remote_files_state::RemoteFilesLocation> for RemoteFilesLocation {
    fn into(self) -> remote_files_state::RemoteFilesLocation {
        remote_files_state::RemoteFilesLocation {
            mount_id: MountId(self.mount_id),
            path: RemotePath(self.path),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesBreadcrumb {
    pub id: String,
    pub mount_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

impl From<&remote_files_state::RemoteFilesBreadcrumb> for RemoteFilesBreadcrumb {
    fn from(breadcrumb: &remote_files_state::RemoteFilesBreadcrumb) -> Self {
        Self {
            id: breadcrumb.id.0.clone(),
            mount_id: breadcrumb.mount_id.0.clone(),
            path: breadcrumb.path.0.clone(),
            name: breadcrumb.name.0.clone(),
            last: breadcrumb.last,
        }
    }
}

// remote_files_browsers

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesBrowserOptions {
    pub select_name: Option<String>,
    pub only_hosted_devices: bool,
}

impl Into<remote_files_browsers_state::RemoteFilesBrowserOptions> for RemoteFilesBrowserOptions {
    fn into(self) -> remote_files_browsers_state::RemoteFilesBrowserOptions {
        remote_files_browsers_state::RemoteFilesBrowserOptions {
            select_name: self.select_name.map(RemoteName),
            only_hosted_devices: self.only_hosted_devices,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RemoteFilesBrowserItemType {
    Bookmarks,
    Place {
        origin: MountOrigin,
    },
    File {
        typ: RemoteFileType,
        file_icon_attrs: FileIconAttrs,
    },
    Shared,
}

impl From<&remote_files_browsers_state::RemoteFilesBrowserItemType> for RemoteFilesBrowserItemType {
    fn from(typ: &remote_files_browsers_state::RemoteFilesBrowserItemType) -> Self {
        match typ {
            remote_files_browsers_state::RemoteFilesBrowserItemType::Bookmarks => Self::Bookmarks,
            remote_files_browsers_state::RemoteFilesBrowserItemType::Place { origin } => {
                Self::Place {
                    origin: origin.into(),
                }
            }
            remote_files_browsers_state::RemoteFilesBrowserItemType::File {
                typ,
                file_icon_attrs,
                ..
            } => Self::File {
                typ: typ.into(),
                file_icon_attrs: file_icon_attrs.into(),
            },
            remote_files_browsers_state::RemoteFilesBrowserItemType::Shared => Self::Shared,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesBrowserItem {
    pub id: String,
    pub mount_id: Option<String>,
    pub path: Option<String>,
    pub name: String,
    pub typ: RemoteFilesBrowserItemType,
    pub size_display: Option<String>,
    pub modified: Option<i64>,
    pub is_selected: bool,
}

impl<'a> From<&remote_files_browsers_state::RemoteFilesBrowserItemInfo<'a>>
    for RemoteFilesBrowserItem
{
    fn from(info: &remote_files_browsers_state::RemoteFilesBrowserItemInfo<'a>) -> Self {
        Self {
            id: info.item.id.0.clone(),
            mount_id: info.item.mount_id.as_ref().map(|x| x.0.clone()),
            path: info.item.path.as_ref().map(|x| x.0.clone()),
            name: info.item.name.0.clone(),
            typ: (&info.item.typ).into(),
            size_display: info
                .item
                .size
                .map(|size| vault_core::files::file_size::size_display(size)),
            modified: info.item.modified,
            is_selected: info.is_selected,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesBrowserInfo {
    pub mount_id: Option<String>,
    pub path: Option<String>,
    pub selection_summary: SelectionSummary,
    pub sort: RemoteFilesSort,
    pub status: Status,
    pub error: Option<String>,
    pub title: Option<String>,
    pub total_count: u32,
    pub total_size_display: String,
    pub selected_count: u32,
    pub selected_size_display: String,
    pub can_create_dir: bool,
    pub items: Vec<RemoteFilesBrowserItem>,
}

impl<'a> From<&remote_files_browsers_state::RemoteFilesBrowserInfo<'a>> for RemoteFilesBrowserInfo {
    fn from(info: &remote_files_browsers_state::RemoteFilesBrowserInfo<'a>) -> Self {
        Self {
            mount_id: info.mount_id.as_ref().map(|x| x.0.clone()),
            path: info.path.as_ref().map(|x| x.0.clone()),
            selection_summary: (&info.selection_summary).into(),
            sort: (&info.sort).into(),
            status: info.status.into(),
            error: match &info.status {
                common_state::Status::Error { error, .. } => Some(error.user_error()),
                _ => None,
            },
            title: info.title.as_ref().map(|x| x.0.clone()),
            total_count: info.total_count as u32,
            total_size_display: vault_core::files::file_size::size_display(info.total_size),
            selected_count: info.selected_count as u32,
            selected_size_display: vault_core::files::file_size::size_display(info.selected_size),
            can_create_dir: info.can_create_dir,
            items: info.items.iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteFilesBrowserBreadcrumb {
    pub id: String,
    pub mount_id: Option<String>,
    pub path: Option<String>,
    pub name: String,
    pub last: bool,
}

impl From<&remote_files_browsers_state::RemoteFilesBrowserBreadcrumb>
    for RemoteFilesBrowserBreadcrumb
{
    fn from(breadcrumb: &remote_files_browsers_state::RemoteFilesBrowserBreadcrumb) -> Self {
        Self {
            id: breadcrumb.id.0.clone(),
            mount_id: breadcrumb.mount_id.as_ref().map(|x| x.0.clone()),
            path: breadcrumb.path.as_ref().map(|x| x.0.clone()),
            name: breadcrumb.name.0.clone(),
            last: breadcrumb.last,
        }
    }
}

pub trait RemoteFilesBrowserDirCreated: Send + Sync + Debug {
    fn on_created(&self, location: String);
}

// repos

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Repo {
    pub id: String,
    pub name: String,
    pub mount_id: String,
    pub path: String,
    pub state: RepoState,
    pub added: i64,
    pub web_url: String,
}

impl From<&repos_state::Repo> for Repo {
    fn from(repo: &repos_state::Repo) -> Self {
        Self {
            id: repo.id.0.clone(),
            name: repo.name.0.clone(),
            mount_id: repo.mount_id.0.clone(),
            path: repo.path.0.clone(),
            state: (&repo.state).into(),
            added: repo.added,
            web_url: repo.web_url.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct RepoConfig {
    pub name: String,
    pub location: RemoteFilesLocation,
    pub password: String,
    pub salt: Option<String>,
    pub rclone_config: String,
}

impl From<&repos_state::RepoConfig> for RepoConfig {
    fn from(config: &repos_state::RepoConfig) -> Self {
        Self {
            name: config.name.0.clone(),
            location: (&config.location).into(),
            password: config.password.clone(),
            salt: config.salt.clone(),
            rclone_config: config.rclone_config.clone(),
        }
    }
}

// repo_create

#[derive(Clone, Debug, PartialEq)]
pub struct RepoCreateForm {
    pub create_load_status: Status,
    pub location: Option<RemoteFilesLocation>,
    pub location_breadcrumbs: Vec<RemoteFilesBreadcrumb>,
    pub password: String,
    pub salt: Option<String>,
    pub fill_from_rclone_config_error: Option<String>,
    pub can_create: bool,
    pub create_repo_status: Status,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepoCreated {
    pub repo_id: String,
    pub config: RepoConfig,
}

impl From<&repos_state::RepoCreated> for RepoCreated {
    fn from(created: &repos_state::RepoCreated) -> Self {
        Self {
            repo_id: created.repo_id.0.clone(),
            config: (&created.config).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RepoCreateInfo {
    Form { form: RepoCreateForm },
    Created { created: RepoCreated },
}

// repo_unlock

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct RepoUnlockInfo {
    pub status: Status,
    pub repo_name: Option<String>,
}

pub trait RepoUnlockUnlocked: Send + Sync + Debug {
    fn on_unlocked(&self);
}

// repo_remove

#[derive(Clone, Debug, PartialEq)]
pub struct RepoRemoveInfo {
    pub status: Status,
    pub repo_name: Option<String>,
}

impl<'a> From<&repo_remove_state::RepoRemoveInfo<'a>> for RepoRemoveInfo {
    fn from(info: &repo_remove_state::RepoRemoveInfo<'a>) -> Self {
        Self {
            status: info.status.into(),
            repo_name: info.repo_name.map(|x| x.0.clone()),
        }
    }
}

pub trait RepoRemoved: Send + Sync + Debug {
    fn on_removed(&self);
}

// repo_files

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFile {
    pub id: String,
    pub repo_id: String,
    pub encrypted_path: String,
    pub decrypted_path: Option<String>,
    pub name: String,
    pub name_error: Option<String>,
    pub ext: Option<String>,
    pub content_type: Option<String>,
    pub typ: RepoFileType,
    pub size_display: String,
    pub modified: Option<i64>,
    pub category: FileCategory,
    pub file_icon_attrs: FileIconAttrs,
}

impl From<&repo_files_state::RepoFile> for RepoFile {
    fn from(file: &repo_files_state::RepoFile) -> Self {
        Self {
            id: file.id.0.clone(),
            repo_id: file.repo_id.0.clone(),
            encrypted_path: file.encrypted_path.0.clone(),
            decrypted_path: match &file.path {
                repo_files_state::RepoFilePath::Decrypted { path } => Some(path.0.clone()),
                repo_files_state::RepoFilePath::DecryptError { .. } => None,
            },
            name: match &file.name {
                repo_files_state::RepoFileName::Decrypted { name, .. } => name.0.clone(),
                repo_files_state::RepoFileName::DecryptError { encrypted_name, .. } => {
                    encrypted_name.0.clone()
                }
            },
            name_error: match &file.name {
                repo_files_state::RepoFileName::Decrypted { .. } => None,
                repo_files_state::RepoFileName::DecryptError { error, .. } => {
                    Some(error.user_error())
                }
            },
            ext: file.ext.clone(),
            content_type: file.content_type.clone(),
            typ: (&file.typ).into(),
            size_display: match &file.size {
                Some(repo_files_state::RepoFileSize::Decrypted { size }) => {
                    vault_core::files::file_size::size_display(*size)
                }
                Some(repo_files_state::RepoFileSize::DecryptError {
                    encrypted_size: _,
                    error: _,
                }) => String::from("???"),
                None => "".into(),
            },
            modified: file.modified,
            category: (&file.category).into(),
            file_icon_attrs: (&file.file_icon_attrs()).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

impl Into<repo_files_state::RepoFilesSortField> for RepoFilesSortField {
    fn into(self) -> repo_files_state::RepoFilesSortField {
        match self {
            Self::Name => repo_files_state::RepoFilesSortField::Name,
            Self::Size => repo_files_state::RepoFilesSortField::Size,
            Self::Modified => repo_files_state::RepoFilesSortField::Modified,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesBreadcrumb {
    pub id: String,
    pub repo_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

impl From<&repo_files_state::RepoFilesBreadcrumb> for RepoFilesBreadcrumb {
    fn from(breadcrumb: &repo_files_state::RepoFilesBreadcrumb) -> Self {
        Self {
            id: breadcrumb.id.0.clone(),
            repo_id: breadcrumb.repo_id.0.clone(),
            path: breadcrumb.path.0.clone(),
            name: breadcrumb.name.clone(),
            last: breadcrumb.last,
        }
    }
}

// transfers

#[derive(Clone, Debug, PartialEq)]
pub enum TransferType {
    Upload,
    Download,
}

impl From<&transfers_state::TransferType> for TransferType {
    fn from(typ: &transfers_state::TransferType) -> Self {
        match typ {
            transfers_state::TransferType::Upload(..) => Self::Upload,
            transfers_state::TransferType::Download => Self::Download,
            transfers_state::TransferType::DownloadReader => Self::Download,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub id: u32,
    pub typ: TransferType,
    pub name: String,
    pub file_icon_attrs: FileIconAttrs,
    pub size: Option<i64>,
    pub size_display: Option<String>,
    pub size_progress_display: Option<String>,
    pub percentage: Option<u8>,
    pub transferred_bytes: i64,
    pub transferred_display: String,
    pub speed_display: Option<String>,
    pub state: TransferState,
    pub can_retry: bool,
    pub can_open: bool,
}

impl From<&transfers_state::Transfer> for Transfer {
    fn from(transfer: &transfers_state::Transfer) -> Self {
        Self {
            id: transfer.id,
            typ: (&transfer.typ).into(),
            name: transfer.name.0.clone(),
            file_icon_attrs: (&transfer.file_icon_attrs()).into(),
            size: transfer.size.exact_or_estimate(),
            size_display: transfer
                .size
                .exact_or_estimate()
                .map(vault_core::files::file_size::size_display),
            size_progress_display: transfer.size.exact_or_estimate().map(|size| {
                vault_core::files::file_size::size_of_display(transfer.transferred_bytes, size)
            }),
            percentage: transfers_selectors::transfer_percentage(transfer),
            transferred_bytes: transfer.transferred_bytes,
            transferred_display: vault_core::files::file_size::size_display(
                transfer.transferred_bytes,
            ),
            speed_display: transfers_selectors::transfer_duration(&transfer, now_ms()).map(
                |duration| {
                    vault_core::files::file_size::speed_display_bytes_duration(
                        transfer.transferred_bytes,
                        duration,
                    )
                },
            ),
            state: (&transfer.state).into(),
            can_retry: transfers_selectors::can_retry(transfer),
            can_open: transfers_selectors::can_open(transfer),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransfersSummary {
    pub total_count: u32,
    pub done_count: u32,
    pub failed_count: u32,
    pub size_progress_display: String,
    pub percentage: u8,
    pub remaining_time_display: String,
    pub speed_display: String,
    pub is_transferring: bool,
    pub is_all_done: bool,
    pub can_retry_all: bool,
    pub can_abort_all: bool,
}

pub trait TransfersDownloadOpen: Send + Sync + Debug {
    fn on_open(&self, local_file_path: String, content_type: Option<String>);
}

pub trait TransfersDownloadDone: Send + Sync + Debug {
    fn on_done(&self, local_file_path: String, content_type: Option<String>);
}

// repo_files_browsers

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesBrowserOptions {
    pub select_name: Option<String>,
}

impl Into<repo_files_browsers_state::RepoFilesBrowserOptions> for RepoFilesBrowserOptions {
    fn into(self) -> repo_files_browsers_state::RepoFilesBrowserOptions {
        repo_files_browsers_state::RepoFilesBrowserOptions {
            select_name: self.select_name.map(DecryptedName),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesBrowserItem {
    pub file: RepoFile,
    pub is_selected: bool,
}

impl<'a> From<&repo_files_browsers_state::RepoFilesBrowserItem<'a>> for RepoFilesBrowserItem {
    fn from(item: &repo_files_browsers_state::RepoFilesBrowserItem<'a>) -> Self {
        Self {
            file: item.file.into(),
            is_selected: item.is_selected,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesBrowserInfo {
    pub repo_id: Option<String>,
    pub encrypted_path: Option<String>,
    pub selection_summary: SelectionSummary,
    pub sort: RepoFilesSort,
    pub status: Status,
    pub error: Option<String>,
    pub title: Option<String>,
    pub total_count: u32,
    pub total_size_display: String,
    pub selected_count: u32,
    pub selected_size_display: String,
    pub items: Vec<RepoFilesBrowserItem>,
}

impl<'a> From<&repo_files_browsers_state::RepoFilesBrowserInfo<'a>> for RepoFilesBrowserInfo {
    fn from(info: &repo_files_browsers_state::RepoFilesBrowserInfo<'a>) -> Self {
        Self {
            repo_id: info.repo_id.map(|x| x.0.clone()),
            encrypted_path: info.path.map(|x| x.0.clone()),
            selection_summary: (&info.selection_summary).into(),
            sort: (&info.sort).into(),
            status: (&info.status).into(),
            error: match &info.status {
                common_state::Status::Error { error, .. } => Some(error.user_error()),
                _ => None,
            },
            title: info.title.as_ref().map(|x| x.clone()),
            total_count: info.total_count as u32,
            total_size_display: vault_core::files::file_size::size_display(info.total_size),
            selected_count: info.selected_count as u32,
            selected_size_display: vault_core::files::file_size::size_display(info.selected_size),
            items: info.items.iter().map(Into::into).collect(),
        }
    }
}

pub trait RepoFilesBrowserDirCreated: Send + Sync + Debug {
    fn on_created(&self, encrypted_path: String);
}

// repo_files_details

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesDetailsOptions {
    pub load_content: FilesFilter,
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

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesDetailsInfo {
    pub repo_id: Option<String>,
    pub encrypted_parent_path: Option<String>,
    pub encrypted_path: Option<String>,
    pub status: Status,
    pub file_name: Option<String>,
    pub file_ext: Option<String>,
    pub file_category: Option<FileCategory>,
    pub file_modified: Option<i64>,
    pub file_exists: bool,
    pub content_status: Status,
    pub transfer_id: Option<u32>,
    pub save_status: Status,
    pub error: Option<String>,
    pub is_editing: bool,
    pub is_dirty: bool,
    pub should_destroy: bool,
    pub can_save: bool,
    pub can_download: bool,
    pub can_copy: bool,
    pub can_move: bool,
    pub can_delete: bool,
}

impl<'a> From<&repo_files_details_state::RepoFilesDetailsInfo<'a>> for RepoFilesDetailsInfo {
    fn from(info: &repo_files_details_state::RepoFilesDetailsInfo<'a>) -> Self {
        Self {
            repo_id: info.repo_id.map(|x| x.0.clone()),
            encrypted_parent_path: info.parent_path.as_ref().map(|x| x.0.clone()),
            encrypted_path: info.path.map(|x| x.0.clone()),
            status: (&info.status).into(),
            file_name: info.file_name.as_ref().map(|x| x.0.clone()),
            file_ext: info.file_ext.clone(),
            file_category: info.file_category.as_ref().map(Into::into),
            file_modified: info.file_modified,
            file_exists: info.file_exists,
            content_status: (&info.content_status).into(),
            transfer_id: info.transfer_id,
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

// repo_files_move

#[derive(Clone, Debug, PartialEq)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

impl From<&repo_files_move_state::RepoFilesMoveMode> for RepoFilesMoveMode {
    fn from(typ: &repo_files_move_state::RepoFilesMoveMode) -> Self {
        match typ {
            repo_files_move_state::RepoFilesMoveMode::Copy => Self::Copy,
            repo_files_move_state::RepoFilesMoveMode::Move => Self::Move,
        }
    }
}

impl Into<repo_files_move_state::RepoFilesMoveMode> for RepoFilesMoveMode {
    fn into(self) -> repo_files_move_state::RepoFilesMoveMode {
        match self {
            Self::Copy => repo_files_move_state::RepoFilesMoveMode::Copy,
            Self::Move => repo_files_move_state::RepoFilesMoveMode::Move,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RepoFilesMoveInfo {
    pub repo_id: String,
    pub src_files_count: u32,
    pub mode: RepoFilesMoveMode,
    pub encrypted_dest_path_chain: Vec<String>,
    pub can_move: bool,
}

// local_files

#[derive(Clone, Debug, PartialEq)]
pub enum LocalFileType {
    Dir,
    File,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalFile {
    pub id: String,
    pub name: String,
    pub ext: Option<String>,
    pub typ: LocalFileType,
    pub size_display: String,
    pub modified: Option<i64>,
    pub category: FileCategory,
    pub file_icon_attrs: FileIconAttrs,
}

// version

#[derive(Clone, Debug, PartialEq)]
pub struct Version {
    pub git_revision: Option<String>,
    pub git_revision_url: Option<String>,
    pub git_release: Option<String>,
    pub git_release_url: Option<String>,
}

impl From<vault_version::Version> for Version {
    fn from(version: vault_version::Version) -> Self {
        Self {
            git_revision: version.git_revision,
            git_revision_url: version.git_revision_url,
            git_release: version.git_release,
            git_release_url: version.git_release_url,
        }
    }
}

// subscription_data

type Data<T> = Arc<Mutex<HashMap<u32, T>>>;

#[derive(Clone)]
struct VersionedFileBytes {
    value: Option<Vec<u8>>,
    version: u32,
}

#[derive(Default)]
struct SubscriptionData {
    notifications: Data<Vec<Notification>>,
    dialogs: Data<Vec<u32>>,
    dialog: Data<Option<Dialog>>,
    oauth2_status: Data<Status>,
    user: Data<Option<User>>,
    user_profile_picture_loaded: Data<bool>,
    remote_files_browsers_info: Data<Option<RemoteFilesBrowserInfo>>,
    remote_files_browsers_breadcrumbs: Data<Vec<RemoteFilesBrowserBreadcrumb>>,
    repos: Data<Repos>,
    repos_repo: Data<RepoInfo>,
    repo_create_info: Data<Option<RepoCreateInfo>>,
    repo_unlock_info: Data<Option<RepoUnlockInfo>>,
    repo_remove_info: Data<Option<RepoRemoveInfo>>,
    repo_files_file: Data<Option<RepoFile>>,
    transfers_is_active: Data<bool>,
    transfers_summary: Data<TransfersSummary>,
    transfers_list: Data<Vec<Transfer>>,
    transfers_transfer: Data<Option<Transfer>>,
    repo_files_browsers_info: Data<Option<RepoFilesBrowserInfo>>,
    repo_files_browsers_breadcrumbs: Data<Vec<RepoFilesBreadcrumb>>,
    repo_files_details_info: Data<Option<RepoFilesDetailsInfo>>,
    repo_files_details_file: Data<Option<RepoFile>>,
    repo_files_details_content_bytes: Data<VersionedFileBytes>,
    repo_files_move_is_visible: Data<bool>,
    repo_files_move_info: Data<Option<RepoFilesMoveInfo>>,
}

lazy_static! {
    static ref RT: Arc<tokio::runtime::Runtime> = Arc::new(tokio::runtime::Runtime::new().unwrap());
}

// logging

pub fn set_logger(level: LoggerLevel, logger: Box<dyn LoggerCallback>) {
    let _ = mobile_logger::init_with_level(level, logger);
}

// MobileVault

pub struct MobileVault {
    pub vault: Arc<Vault>,
    pub version: vault_version::Version,
    pub reqwest_client: Arc<reqwest::Client>,
    pub tokio_tungstenite_connector: Option<tokio_tungstenite::Connector>,
    pub errors: Arc<MobileErrors>,
    pub spawn: Arc<MobileSpawn>,
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
    pub file_icon_factory: vault_file_icon::FileIconFactory,

    subscription_data: SubscriptionData,
    subscription: MobileSubscription,
}

impl MobileVault {
    pub fn new(
        base_url: String,
        app_name: String,
        oauth2_auth_base_url: String,
        oauth2_client_id: String,
        oauth2_client_secret: String,
        oauth2_redirect_uri: String,
        secure_storage: Box<dyn SecureStorage>,
    ) -> Self {
        mobile_logger::try_init_env_logger();

        Self::new_with_options(
            base_url,
            app_name,
            oauth2_auth_base_url,
            oauth2_client_id,
            oauth2_client_secret,
            oauth2_redirect_uri,
            secure_storage,
            RT.clone(),
        )
    }

    pub fn new_with_options(
        base_url: String,
        app_name: String,
        oauth2_auth_base_url: String,
        oauth2_client_id: String,
        oauth2_client_secret: String,
        oauth2_redirect_uri: String,
        secure_storage: Box<dyn SecureStorage>,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    ) -> Self {
        let version = vault_version::Version::new();

        let version_info = version
            .git_release
            .as_deref()
            .or(version.git_revision.as_deref())
            .unwrap_or("unknown");

        let user_agent = format!("{}/{}", app_name, version_info);

        let oauth2_config = OAuth2Config {
            base_url: base_url.clone(),
            auth_base_url: oauth2_auth_base_url.clone(),
            client_id: oauth2_client_id,
            client_secret: oauth2_client_secret,
            redirect_uri: oauth2_redirect_uri,
        };

        let secure_storage = Box::new(MobileSecureStorage::new(secure_storage));

        let (vault, reqwest_client, tokio_tungstenite_connector) = build_vault(
            base_url,
            user_agent,
            oauth2_config,
            secure_storage,
            tokio_runtime.clone(),
        );

        let errors = Arc::new(MobileErrors::new(vault.clone()));
        let spawn = Arc::new(MobileSpawn::new(tokio_runtime.clone(), errors.clone()));

        let subscription_data = SubscriptionData::default();
        let subscription = MobileSubscription::new(vault.clone());

        let file_icon_theme = vault_file_icon::FileIconTheme::default();
        let file_icon_factory = vault_file_icon::FileIconFactory::new(&file_icon_theme);

        Self {
            vault,
            version,
            reqwest_client,
            tokio_tungstenite_connector,
            errors,
            spawn,
            tokio_runtime,
            file_icon_factory,

            subscription_data,
            subscription,
        }
    }

    // spawn

    fn spawn(self: Arc<Self>, future: impl Future<Output = ()> + Send + 'static) {
        self.spawn.clone().spawn(future);
    }

    fn spawn_result(
        self: Arc<Self>,
        future: impl Future<Output = Result<(), impl UserError>> + Send + 'static,
    ) {
        self.spawn.clone().spawn_result(future);
    }

    fn spawn_blocking(&self, f: impl FnOnce() + Send + 'static) {
        self.spawn.spawn_blocking(f);
    }

    // subscription

    fn subscribe<T: Clone + PartialEq + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        callback: Box<dyn SubscriptionCallback>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>) -> T + Send + Sync + 'static,
    ) -> u32 {
        self.subscription
            .subscribe(events, callback, subscription_data, generate_data)
    }

    fn subscribe_changed<T: Clone + Send + 'static>(
        &self,
        events: &[vault_core::store::Event],
        callback: Box<dyn SubscriptionCallback>,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
        generate_data: impl Fn(Arc<vault_core::Vault>, hash_map::Entry<'_, u32, T>) -> bool
            + Send
            + Sync
            + 'static,
    ) -> u32 {
        self.subscription
            .subscribe_changed(events, callback, subscription_data, generate_data)
    }

    fn get_data<T: Clone + Send>(
        &self,
        id: u32,
        subscription_data: Arc<Mutex<HashMap<u32, T>>>,
    ) -> Option<T> {
        self.subscription.get_data(id, subscription_data)
    }

    pub fn unsubscribe(&self, id: u32) {
        self.subscription.unsubscribe(id)
    }

    // lifecycle

    pub fn load(self: Arc<Self>) {
        self.clone()
            .spawn_result(async move { self.vault.load().await })
    }

    pub fn logout(&self) {
        self.errors.handle_result(self.vault.logout());
    }

    // relative_time

    pub fn relative_time(&self, value: i64, with_modifier: bool) -> RelativeTime {
        self.vault.relative_time(value, with_modifier).into()
    }

    // notifications

    pub fn notifications_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
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

    pub fn notifications_data(&self, id: u32) -> Option<Vec<Notification>> {
        self.get_data(id, self.subscription_data.notifications.clone())
    }

    pub fn notifications_show(&self, message: String) {
        self.vault.notifications_show(message)
    }

    pub fn notifications_remove(&self, notification_id: u32) {
        self.vault.notifications_remove(notification_id)
    }

    pub fn notifications_remove_after(self: Arc<Self>, notification_id: u32, duration_ms: u32) {
        self.clone().spawn.spawn(async move {
            self.vault
                .notifications_remove_after(
                    notification_id,
                    Duration::from_millis(duration_ms as u64),
                )
                .await
        })
    }

    // dialogs

    pub fn dialogs_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::Dialogs],
            cb,
            self.subscription_data.dialogs.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::dialogs::selectors::select_dialogs(state)
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

    pub fn dialogs_dialog_subscribe(
        &self,
        dialog_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::Dialogs],
            cb,
            self.subscription_data.dialog.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::dialogs::selectors::select_dialog_info(state, dialog_id)
                        .map(Into::into)
                })
            },
        )
    }

    pub fn dialogs_dialog_data(&self, id: u32) -> Option<Dialog> {
        self.get_data(id, self.subscription_data.dialog.clone())
            .flatten()
    }

    pub fn dialogs_confirm(&self, dialog_id: u32) {
        self.vault.dialogs_confirm(dialog_id)
    }

    pub fn dialogs_cancel(&self, dialog_id: u32) {
        self.vault.dialogs_cancel(dialog_id)
    }

    pub fn dialogs_set_input_value(&self, dialog_id: u32, value: String) {
        self.vault.dialogs_set_input_value(dialog_id, value);
    }

    // oauth2

    pub fn oauth2_status_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::Auth],
            cb,
            self.subscription_data.oauth2_status.clone(),
            move |vault| {
                vault.with_state(|state| vault_core::oauth2::selectors::select_status(state).into())
            },
        )
    }

    pub fn oauth2_status_data(&self, id: u32) -> Option<Status> {
        self.get_data(id, self.subscription_data.oauth2_status.clone())
    }

    pub fn oauth2_start_login_flow(&self) -> Option<String> {
        match self.vault.oauth2_start_login_flow() {
            Ok(url) => Some(url),
            Err(err) => {
                self.errors.handle_error(err);
                None
            }
        }
    }

    pub fn oauth2_start_logout_flow(&self) -> Option<String> {
        match self.vault.oauth2_start_logout_flow() {
            Ok(url) => Some(url),
            Err(err) => {
                self.errors.handle_error(err);
                None
            }
        }
    }

    pub fn oauth2_finish_flow_url(self: Arc<Self>, url: String, cb: Box<dyn OAuth2FinishFlowDone>) {
        self.clone().spawn_result(async move {
            let res = self.vault.oauth2_finish_flow_url(&url).await;

            cb.on_done();

            res
        })
    }

    // user

    pub fn user_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::User],
            cb,
            self.subscription_data.user.clone(),
            move |vault| vault.with_state(|state| state.user.user.as_ref().map(Into::into)),
        )
    }

    pub fn user_data(&self, id: u32) -> Option<User> {
        self.get_data(id, self.subscription_data.user.clone())
            .flatten()
    }

    pub fn user_profile_picture_loaded_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
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
                            common_state::Status::Loaded => true,
                            _ => false,
                        })
                        .unwrap_or(false)
                })
            },
        )
    }

    pub fn user_profile_picture_loaded_data(&self, id: u32) -> Option<bool> {
        self.get_data(
            id,
            self.subscription_data.user_profile_picture_loaded.clone(),
        )
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

    pub fn user_ensure_profile_picture(self: Arc<Self>) {
        self.clone()
            .spawn_result(async move { self.vault.user_ensure_profile_picture().await })
    }

    // file_icon

    pub fn file_icon_png(&self, props: FileIconProps, scale: u32) -> FileIconPng {
        let (svg, width, height) = self.file_icon_factory.generate_svg(&props.into());

        let width = width * scale;
        let height = height * scale;

        let png = vault_file_icon::render_png(&svg, width, height).unwrap();

        FileIconPng { png, width, height }
    }

    // remote_files_browsers

    pub fn remote_files_browsers_create(
        self: Arc<Self>,
        location: String,
        options: RemoteFilesBrowserOptions,
    ) -> u32 {
        let (browser_id, load_future) = self
            .vault
            .remote_files_browsers_create(&RemoteFilesBrowserItemId(location), options.into());

        self.clone().spawn_result(async move { load_future.await });

        browser_id
    }

    pub fn remote_files_browsers_destroy(&self, browser_id: u32) {
        self.vault.remote_files_browsers_destroy(browser_id)
    }

    pub fn remote_files_browsers_info_subscribe(
        &self,
        browser_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RemoteFilesBrowsers, Event::RemoteFiles],
            cb,
            self.subscription_data.remote_files_browsers_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::remote_files_browsers::selectors::select_info(state, browser_id)
                        .as_ref()
                        .map(RemoteFilesBrowserInfo::from)
                })
            },
        )
    }

    pub fn remote_files_browsers_info_data(&self, id: u32) -> Option<RemoteFilesBrowserInfo> {
        self.get_data(
            id,
            self.subscription_data.remote_files_browsers_info.clone(),
        )
        .flatten()
    }

    pub fn remote_files_browsers_breadcrumbs_subscribe(
        &self,
        browser_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RemoteFilesBrowsers],
            cb,
            self.subscription_data
                .remote_files_browsers_breadcrumbs
                .clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::remote_files_browsers::selectors::select_breadcrumbs(
                        state, browser_id,
                    )
                    .iter()
                    .map(Into::into)
                    .collect()
                })
            },
        )
    }

    pub fn remote_files_browsers_breadcrumbs_data(
        &self,
        id: u32,
    ) -> Option<Vec<RemoteFilesBrowserBreadcrumb>> {
        self.get_data(
            id,
            self.subscription_data
                .remote_files_browsers_breadcrumbs
                .clone(),
        )
    }

    pub fn remote_files_browsers_load(self: Arc<Self>, browser_id: u32) {
        self.clone()
            .spawn_result(async move { self.vault.remote_files_browsers_load(browser_id).await })
    }

    pub fn remote_files_browsers_select_item(
        &self,
        browser_id: u32,
        item_id: String,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.vault.remote_files_browsers_select_item(
            browser_id,
            RemoteFilesBrowserItemId(item_id),
            extend,
            range,
            force,
        )
    }

    pub fn remote_files_browsers_select_all(&self, browser_id: u32) {
        self.vault.remote_files_browsers_select_all(browser_id)
    }

    pub fn remote_files_browsers_clear_selection(&self, browser_id: u32) {
        self.vault.remote_files_browsers_clear_selection(browser_id)
    }

    pub fn remote_files_browsers_set_selection(&self, browser_id: u32, selection: Vec<String>) {
        self.vault.remote_files_browsers_set_selection(
            browser_id,
            selection
                .into_iter()
                .map(RemoteFilesBrowserItemId)
                .collect(),
        )
    }

    pub fn remote_files_browsers_sort_by(
        &self,
        browser_id: u32,
        field: RemoteFilesSortField,
        direction: Option<SortDirection>,
    ) {
        self.vault.remote_files_browsers_sort_by(
            browser_id,
            field.into(),
            direction.map(Into::into),
        )
    }

    pub fn remote_files_browsers_create_dir(
        self: Arc<Self>,
        browser_id: u32,
        cb: Box<dyn RemoteFilesBrowserDirCreated>,
    ) {
        self.clone().spawn_result(async move {
            match self
                .vault
                .remote_files_browsers_create_dir(browser_id)
                .await
            {
                Ok(location) => {
                    cb.on_created(location.0);

                    Ok(())
                }
                Err(vault_core::remote_files::errors::CreateDirError::Canceled) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    // repos

    pub fn repos_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::Repos],
            cb,
            self.subscription_data.repos.clone(),
            move |vault| vault.with_state(|state| Repos::from(state)),
        )
    }

    pub fn repos_data(&self, id: u32) -> Option<Repos> {
        self.get_data(id, self.subscription_data.repos.clone())
    }

    pub fn repos_load(self: Arc<Self>) {
        self.clone()
            .spawn_result(async move { self.vault.repos_load().await })
    }

    pub fn repos_repo_subscribe(&self, repo_id: String, cb: Box<dyn SubscriptionCallback>) -> u32 {
        let repo_id = RepoId(repo_id);

        self.subscribe(
            &[Event::Repos],
            cb,
            self.subscription_data.repos_repo.clone(),
            move |vault| {
                vault.with_state(|state| {
                    (&vault_core::repos::selectors::select_repo_info(state, &repo_id)).into()
                })
            },
        )
    }

    pub fn repos_repo_data(&self, id: u32) -> Option<RepoInfo> {
        self.get_data(id, self.subscription_data.repos_repo.clone())
    }

    pub fn repos_lock_repo(&self, repo_id: String) {
        self.errors
            .handle_result(self.vault.repos_lock_repo(&RepoId(repo_id)));
    }

    // repo_create

    pub fn repo_create_create(self: Arc<Self>) -> u32 {
        let (create_id, create_load_future) = self.vault.repo_create_create();

        self.clone().spawn(async move {
            // error is displayed in the create component
            let _ = create_load_future.await;
        });

        create_id
    }

    pub fn repo_create_create_load(self: Arc<Self>, create_id: u32) {
        self.clone().spawn(async move {
            // error is displayed in the create component
            let _ = self.vault.repo_create_create_load(create_id).await;
        });
    }

    pub fn repo_create_info_subscribe(
        &self,
        create_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            // TODO remove dir pickers
            &[Event::RepoCreate, Event::DirPickers],
            cb,
            self.subscription_data.repo_create_info.clone(),
            move |vault| {
                use vault_core::{
                    remote_files::selectors as remote_files_selectors,
                    repo_create::{selectors, state::RepoCreate},
                };

                vault.with_state(|state| {
                    vault_core::repo_create::selectors::select_repo_create(state, create_id).map(
                        |repo_create| match repo_create {
                            vault_core::repo_create::state::RepoCreate::Form(form) => {
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

                                let location =
                                    form.location.as_ref().map(|location| location.into());
                                let location_breadcrumbs = location_breadcrumbs
                                    .iter()
                                    .map(RemoteFilesBreadcrumb::from)
                                    .collect();
                                let password = form.password.clone();
                                let salt = form.salt.clone();
                                let fill_from_rclone_config_error = form
                                    .fill_from_rclone_config_error
                                    .as_ref()
                                    .map(|e| e.to_string());
                                let can_create = selectors::select_can_create(state, create_id);
                                let create_repo_status = (&form.create_repo_status).into();

                                RepoCreateInfo::Form {
                                    form: RepoCreateForm {
                                        create_load_status: (&form.create_load_status).into(),
                                        location,
                                        location_breadcrumbs,
                                        password,
                                        salt,
                                        fill_from_rclone_config_error,
                                        can_create,
                                        create_repo_status,
                                    },
                                }
                            }
                            RepoCreate::Created(created) => RepoCreateInfo::Created {
                                created: created.into(),
                            },
                        },
                    )
                })
            },
        )
    }

    pub fn repo_create_info_data(&self, id: u32) -> Option<RepoCreateInfo> {
        self.get_data(id, self.subscription_data.repo_create_info.clone())
            .flatten()
    }

    pub fn repo_create_set_location(&self, create_id: u32, location: RemoteFilesLocation) {
        self.vault
            .repo_create_set_location(create_id, location.into())
    }

    pub fn repo_create_set_password(&self, create_id: u32, password: String) {
        self.vault.repo_create_set_password(create_id, password)
    }

    pub fn repo_create_set_salt(&self, create_id: u32, salt: Option<String>) {
        self.vault.repo_create_set_salt(create_id, salt)
    }

    pub fn repo_create_fill_from_rclone_config(&self, create_id: u32, config: String) -> bool {
        self.vault
            .repo_create_fill_from_rclone_config(create_id, config)
            .is_ok()
    }

    pub fn repo_create_create_repo(self: Arc<Self>, create_id: u32) {
        self.clone().spawn(async move {
            let _ = self.vault.repo_create_create_repo(create_id).await;
        });
    }

    pub fn repo_create_destroy(&self, create_id: u32) {
        self.vault.repo_create_destroy(create_id);
    }

    // repo_unlock

    pub fn repo_unlock_info_subscribe(
        &self,
        repo_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoUnlock],
            cb,
            self.subscription_data.repo_unlock_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_unlock::selectors::select_info(state, repo_id).map(|info| {
                        RepoUnlockInfo {
                            status: info.status.into(),
                            repo_name: info.repo_name.map(|x| x.0.clone()),
                        }
                    })
                })
            },
        )
    }

    pub fn repo_unlock_info_data(&self, id: u32) -> Option<RepoUnlockInfo> {
        self.get_data(id, self.subscription_data.repo_unlock_info.clone())
            .flatten()
    }

    pub fn repo_unlock_create(&self, repo_id: String, options: RepoUnlockOptions) -> u32 {
        self.vault
            .repo_unlock_create(RepoId(repo_id), options.into())
    }

    pub fn repo_unlock_unlock(
        self: Arc<Self>,
        unlock_id: u32,
        password: String,
        cb: Box<dyn RepoUnlockUnlocked>,
    ) {
        // use a thread pool, unlock takes a while and would block UI
        self.clone().spawn_blocking(move || {
            match self.vault.repo_unlock_unlock(unlock_id, &password) {
                Ok(()) => {
                    cb.on_unlocked();
                }
                _ => {}
            }
        })
    }

    pub fn repo_unlock_destroy(&self, unlock_id: u32) {
        self.vault.repo_unlock_destroy(unlock_id)
    }

    // repo_remove

    pub fn repo_remove_create(&self, repo_id: String) -> u32 {
        self.vault.repo_remove_create(RepoId(repo_id))
    }

    pub fn repo_remove_info_subscribe(
        &self,
        remove_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoRemove],
            cb,
            self.subscription_data.repo_remove_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_remove::selectors::select_info(state, remove_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_remove_info_data(&self, id: u32) -> Option<RepoRemoveInfo> {
        self.get_data(id, self.subscription_data.repo_remove_info.clone())
            .flatten()
    }

    pub fn repo_remove_remove(
        self: Arc<Self>,
        remove_id: u32,
        password: String,
        cb: Box<dyn RepoRemoved>,
    ) {
        self.clone().spawn(async move {
            if self
                .vault
                .repo_remove_remove(remove_id, &password)
                .await
                .is_ok()
            {
                cb.on_removed();
            }
        });
    }

    pub fn repo_remove_destroy(&self, remove_id: u32) {
        self.vault.repo_remove_destroy(remove_id)
    }

    // repo_files

    pub fn repo_files_file_subscribe(
        &self,
        file_id: String,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        let file_id = RepoFileId(file_id);

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

    pub fn repo_files_file_data(&self, id: u32) -> Option<RepoFile> {
        self.get_data(id, self.subscription_data.repo_files_file.clone())
            .flatten()
    }

    pub fn repo_files_delete_file(self: Arc<Self>, repo_id: String, encrypted_path: String) {
        self.clone().spawn_result(async move {
            match self
                .vault
                .repo_files_delete_files(&[(RepoId(repo_id), EncryptedPath(encrypted_path))])
                .await
            {
                Ok(()) => Ok(()),
                Err(vault_core::repo_files::errors::DeleteFileError::Canceled) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    pub fn repo_files_rename_file(self: Arc<Self>, repo_id: String, encrypted_path: String) {
        self.clone().spawn_result(async move {
            self.vault
                .repo_files_rename_file(&RepoId(repo_id), &EncryptedPath(encrypted_path))
                .await
        })
    }

    pub fn repo_files_move_file(
        self: Arc<Self>,
        repo_id: String,
        encrypted_path: String,
        mode: RepoFilesMoveMode,
    ) {
        self.clone().spawn_result(async move {
            self.vault
                .repo_files_move_move_file(
                    RepoId(repo_id),
                    EncryptedPath(encrypted_path),
                    mode.into(),
                )
                .await
        })
    }

    // transfers

    pub fn transfers_is_active_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_is_active.clone(),
            move |vault| vault.with_state(|state| transfers::selectors::select_is_active(state)),
        )
    }

    pub fn transfers_is_active_data(&self, id: u32) -> Option<bool> {
        self.get_data(id, self.subscription_data.transfers_is_active.clone())
    }

    pub fn transfers_summary_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_summary.clone(),
            move |vault| {
                vault.with_state(|state| {
                    use transfers::selectors;

                    let now = now_ms();

                    TransfersSummary {
                        total_count: state.transfers.total_count as u32,
                        done_count: state.transfers.done_count as u32,
                        failed_count: state.transfers.failed_count as u32,
                        size_progress_display: vault_core::files::file_size::size_of_display(
                            state.transfers.done_bytes,
                            state.transfers.total_bytes,
                        ),
                        percentage: selectors::select_percentage(state),
                        remaining_time_display: selectors::select_remaining_time(state, now)
                            .to_string(),
                        speed_display: vault_core::files::file_size::speed_display_bytes_duration(
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

    pub fn transfers_summary_data(&self, id: u32) -> Option<TransfersSummary> {
        self.get_data(id, self.subscription_data.transfers_summary.clone())
    }

    pub fn transfers_list_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_list.clone(),
            move |vault| {
                vault.with_state(|state| {
                    transfers::selectors::select_transfers(state)
                        .into_iter()
                        .map(Into::into)
                        .collect()
                })
            },
        )
    }

    pub fn transfers_list_data(&self, id: u32) -> Option<Vec<Transfer>> {
        self.get_data(id, self.subscription_data.transfers_list.clone())
    }

    pub fn transfers_transfer_subscribe(
        &self,
        transfer_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::Transfers],
            cb,
            self.subscription_data.transfers_transfer.clone(),
            move |vault| {
                vault.with_state(|state| {
                    transfers::selectors::select_transfer(state, transfer_id).map(Into::into)
                })
            },
        )
    }

    pub fn transfers_transfer_data(&self, id: u32) -> Option<Transfer> {
        self.get_data(id, self.subscription_data.transfers_transfer.clone())
            .flatten()
    }

    pub fn transfers_upload_file(
        self: Arc<Self>,
        repo_id: String,
        encrypted_parent_path: String,
        name: String,
        local_file_path: String,
        remove_file_after_upload: bool,
    ) {
        let uploadable = Box::new(MobileUploadable::File {
            path: local_file_path,
            remove_file_after_upload,
            tokio_runtime: self.tokio_runtime.clone(),
        });

        self.clone().transfers_upload_uploadable(
            RepoId(repo_id),
            EncryptedPath(encrypted_parent_path),
            transfers::state::TransferUploadRelativeName(name),
            uploadable,
        );
    }

    pub fn transfers_upload_stream(
        self: Arc<Self>,
        repo_id: String,
        encrypted_parent_path: String,
        name: String,
        stream_provider: Box<dyn UploadStreamProvider>,
    ) {
        let uploadable = Box::new(MobileUploadable::Stream {
            stream_provider: Arc::new(stream_provider),
            tokio_runtime: self.tokio_runtime.clone(),
        });

        self.clone().transfers_upload_uploadable(
            RepoId(repo_id),
            EncryptedPath(encrypted_parent_path),
            transfers::state::TransferUploadRelativeName(name),
            uploadable,
        );
    }

    pub fn transfers_upload_bytes(
        self: Arc<Self>,
        repo_id: String,
        parent_path: String,
        name: String,
        bytes: Vec<u8>,
    ) {
        let uploadable = Box::new(MobileUploadable::Bytes { bytes });

        self.clone().transfers_upload_uploadable(
            RepoId(repo_id),
            EncryptedPath(parent_path),
            transfers::state::TransferUploadRelativeName(name),
            uploadable,
        );
    }

    fn transfers_upload_uploadable(
        self: Arc<Self>,
        repo_id: RepoId,
        parent_path: EncryptedPath,
        name: transfers::state::TransferUploadRelativeName,
        uploadable: transfers::uploadable::BoxUploadable,
    ) {
        self.clone().spawn(async move {
            let (_, create_future) =
                self.vault
                    .transfers_upload(repo_id, parent_path, name, uploadable);

            let future = match create_future.await {
                Ok(future) => future,
                Err(err) => {
                    self.errors.handle_error(err);
                    return;
                }
            };

            // errors are displayed in transfers
            let _ = future.await;
        });
    }

    pub fn transfers_download_file(
        self: Arc<Self>,
        repo_id: String,
        encrypted_path: String,
        local_file_path: String,
        append_name: bool,
        autorename: bool,
        on_open: Option<Box<dyn TransfersDownloadOpen>>,
        on_done: Box<dyn TransfersDownloadDone>,
    ) {
        self.transfers_download(
            RepoId(repo_id),
            EncryptedPath(encrypted_path),
            Box::new(MobileDownloadable::File {
                original_path: local_file_path.into(),
                append_name,
                autorename,
                on_open,
                on_done,
                path: None,
                content_type: None,
            }),
        );
    }

    pub fn transfers_download_temp_file(
        self: Arc<Self>,
        repo_id: String,
        encrypted_path: String,
        local_base_path: String,
        on_open: Option<Box<dyn TransfersDownloadOpen>>,
        on_done: Box<dyn TransfersDownloadDone>,
    ) {
        self.transfers_download(
            RepoId(repo_id),
            EncryptedPath(encrypted_path),
            Box::new(MobileDownloadable::TempFile {
                base_path: local_base_path.into(),
                on_open,
                on_done,
                parent_path: None,
                temp_path: None,
                path: None,
                content_type: None,
            }),
        );
    }

    pub fn transfers_download_stream(
        self: Arc<Self>,
        repo_id: String,
        encrypted_path: String,
        stream_provider: Box<dyn DownloadStreamProvider>,
    ) {
        self.clone().transfers_download(
            RepoId(repo_id),
            EncryptedPath(encrypted_path),
            Box::new(MobileDownloadable::Stream {
                stream_provider: Arc::new(stream_provider),
                tokio_runtime: self.tokio_runtime.clone(),
            }),
        );
    }

    fn transfers_download(
        self: Arc<Self>,
        repo_id: RepoId,
        path: EncryptedPath,
        downloadable: downloadable::BoxDownloadable,
    ) {
        self.clone().spawn(async move {
            let reader_provider = match self.vault.repo_files_get_file_reader(&repo_id, &path) {
                Ok(reader_provider) => reader_provider,
                Err(err) => {
                    self.errors.handle_error(err);
                    return;
                }
            };

            self.transfers_download_reader_provider_downloadable(reader_provider, downloadable)
                .await;
        });
    }

    async fn transfers_download_reader_provider_downloadable(
        self: Arc<Self>,
        reader_provider: repo_files_read::state::RepoFileReaderProvider,
        downloadable: downloadable::BoxDownloadable,
    ) {
        let (_, create_future) = self.vault.transfers_download(reader_provider, downloadable);

        self.transfers_process_create_download_result_future(create_future)
            .await;
    }

    async fn transfers_process_create_download_result_future(
        self: Arc<Self>,
        create_future: transfers_state::CreateDownloadResultFuture,
    ) {
        let future = match create_future.await {
            Ok(future) => future,
            Err(err) if matches!(err, TransferError::AlreadyExists) => return,
            Err(err) => {
                self.errors.handle_error(err);
                return;
            }
        };

        // errors are displayed in transfers
        let _ = future.await;
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

    pub fn transfers_open(self: Arc<Self>, id: u32) {
        self.clone()
            .spawn_result(async move { self.vault.transfers_open(id).await });
    }

    // repo_files_browsers

    pub fn repo_files_browsers_create(
        self: Arc<Self>,
        repo_id: String,
        encrypted_path: String,
        options: RepoFilesBrowserOptions,
    ) -> u32 {
        let (browser_id, load_future) = self.vault.repo_files_browsers_create(
            RepoId(repo_id),
            &EncryptedPath(encrypted_path),
            options.into(),
        );

        self.clone().spawn_result(async move { load_future.await });

        browser_id
    }

    pub fn repo_files_browsers_destroy(&self, browser_id: u32) {
        self.vault.repo_files_browsers_destroy(browser_id)
    }

    pub fn repo_files_browsers_info_subscribe(
        &self,
        browser_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoFilesBrowsers, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_browsers_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_browsers::selectors::select_info(state, browser_id)
                        .as_ref()
                        .map(RepoFilesBrowserInfo::from)
                })
            },
        )
    }

    pub fn repo_files_browsers_info_data(&self, id: u32) -> Option<RepoFilesBrowserInfo> {
        self.get_data(id, self.subscription_data.repo_files_browsers_info.clone())
            .flatten()
    }

    pub fn repo_files_browsers_breadcrumbs_subscribe(
        &self,
        browser_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoFilesBrowsers],
            cb,
            self.subscription_data
                .repo_files_browsers_breadcrumbs
                .clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_browsers::selectors::select_info(state, browser_id)
                        .as_ref()
                        .and_then(|info| {
                            info.breadcrumbs
                                .map(|breadcrumbs| breadcrumbs.iter().map(Into::into).collect())
                        })
                        .unwrap_or(vec![])
                })
            },
        )
    }

    pub fn repo_files_browsers_breadcrumbs_data(
        &self,
        id: u32,
    ) -> Option<Vec<RepoFilesBreadcrumb>> {
        self.get_data(
            id,
            self.subscription_data
                .repo_files_browsers_breadcrumbs
                .clone(),
        )
    }

    pub fn repo_files_browsers_load_files(self: Arc<Self>, browser_id: u32) {
        self.clone().spawn_result(async move {
            self.vault.repo_files_browsers_load_files(browser_id).await
        })
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
        )
    }

    pub fn repo_files_browsers_select_all(&self, browser_id: u32) {
        self.vault.repo_files_browsers_select_all(browser_id)
    }

    pub fn repo_files_browsers_clear_selection(&self, browser_id: u32) {
        self.vault.repo_files_browsers_clear_selection(browser_id)
    }

    pub fn repo_files_browsers_set_selection(&self, browser_id: u32, selection: Vec<String>) {
        self.vault.repo_files_browsers_set_selection(
            browser_id,
            selection.into_iter().map(RepoFileId).collect(),
        )
    }

    pub fn repo_files_browsers_sort_by(
        &self,
        browser_id: u32,
        field: RepoFilesSortField,
        direction: Option<SortDirection>,
    ) {
        self.vault
            .repo_files_browsers_sort_by(browser_id, field.into(), direction.map(Into::into))
    }

    pub fn repo_files_browsers_download_selected_file(
        self: Arc<Self>,
        browser_id: u32,
        local_file_path: String,
        append_name: bool,
        autorename: bool,
        on_open: Option<Box<dyn TransfersDownloadOpen>>,
        on_done: Box<dyn TransfersDownloadDone>,
    ) {
        self.clone().spawn(async move {
            let reader_provider = match self
                .vault
                .clone()
                .repo_files_browsers_get_selected_reader(browser_id)
            {
                Ok(reader_provider) => reader_provider,
                Err(err) => {
                    self.errors.handle_error(err);
                    return;
                }
            };

            let downloadable = Box::new(MobileDownloadable::File {
                original_path: local_file_path.into(),
                append_name,
                autorename,
                on_open,
                on_done,
                path: None,
                content_type: None,
            });

            self.transfers_download_reader_provider_downloadable(reader_provider, downloadable)
                .await;
        });
    }

    pub fn repo_files_browsers_download_selected_stream(
        self: Arc<Self>,
        browser_id: u32,
        stream_provider: Box<dyn DownloadStreamProvider>,
    ) {
        self.clone().spawn(async move {
            let reader_provider = match self
                .vault
                .clone()
                .repo_files_browsers_get_selected_reader(browser_id)
            {
                Ok(reader_provider) => reader_provider,
                Err(err) => {
                    self.errors.handle_error(err);
                    return;
                }
            };

            let downloadable = Box::new(MobileDownloadable::Stream {
                stream_provider: Arc::new(stream_provider),
                tokio_runtime: self.tokio_runtime.clone(),
            });

            self.transfers_download_reader_provider_downloadable(reader_provider, downloadable)
                .await;
        });
    }

    pub fn repo_files_browsers_create_dir(
        self: Arc<Self>,
        browser_id: u32,
        cb: Box<dyn RepoFilesBrowserDirCreated>,
    ) {
        self.clone().spawn_result(async move {
            match self.vault.repo_files_browsers_create_dir(browser_id).await {
                Ok((_, path)) => {
                    cb.on_created(path.0);

                    Ok(())
                }
                Err(vault_core::repo_files::errors::CreateDirError::Canceled) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    pub fn repo_files_browsers_delete_selected(self: Arc<Self>, browser_id: u32) {
        self.clone().spawn_result(async move {
            match self
                .vault
                .repo_files_browsers_delete_selected(browser_id)
                .await
            {
                Ok(()) => Ok(()),
                Err(vault_core::repo_files::errors::DeleteFileError::Canceled) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    pub fn repo_files_browsers_move_selected(
        self: Arc<Self>,
        browser_id: u32,
        mode: RepoFilesMoveMode,
    ) {
        self.clone().spawn_result(async move {
            self.vault
                .repo_files_browsers_move_selected(browser_id, mode.into())
                .await
        })
    }

    // repo_files_details

    pub fn repo_files_details_create(
        self: Arc<Self>,
        repo_id: String,
        encrypted_path: String,
        is_editing: bool,
        options: RepoFilesDetailsOptions,
    ) -> u32 {
        let (details_id, load_future) = self.vault.repo_files_details_create(
            RepoId(repo_id),
            &EncryptedPath(encrypted_path),
            is_editing,
            options.into(),
        );

        self.clone().spawn(async move {
            // error is displayed in the details component
            let _ = load_future.await;
        });

        details_id
    }

    pub fn repo_files_details_destroy(self: Arc<Self>, details_id: u32) {
        self.clone().spawn_result(async move {
            self.vault
                .clone()
                .repo_files_details_destroy(details_id)
                .await
        });
    }

    pub fn repo_files_details_info_subscribe(
        &self,
        details_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoFilesDetails, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_details_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_details::selectors::select_info(state, details_id)
                        .as_ref()
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_files_details_info_data(&self, id: u32) -> Option<RepoFilesDetailsInfo> {
        self.get_data(id, self.subscription_data.repo_files_details_info.clone())
            .flatten()
    }

    pub fn repo_files_details_file_subscribe(
        &self,
        details_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe(
            &[Event::RepoFilesDetails, Event::RepoFiles],
            cb,
            self.subscription_data.repo_files_details_file.clone(),
            move |vault| {
                vault.with_state(|state| {
                    vault_core::repo_files_details::selectors::select_file(state, details_id)
                        .map(Into::into)
                })
            },
        )
    }

    pub fn repo_files_details_file_data(&self, id: u32) -> Option<RepoFile> {
        self.get_data(id, self.subscription_data.repo_files_details_file.clone())
            .flatten()
    }

    pub fn repo_files_details_content_bytes_subscribe(
        &self,
        details_id: u32,
        cb: Box<dyn SubscriptionCallback>,
    ) -> u32 {
        self.subscribe_changed(
            &[Event::RepoFilesDetailsContentData],
            cb,
            self.subscription_data
                .repo_files_details_content_bytes
                .clone(),
            move |vault, entry| {
                vault.with_state(|state| {
                    let (bytes, version) =
                        vault_core::repo_files_details::selectors::select_content_bytes_version(
                            state, details_id,
                        );

                    vault_core::store::update_if(
                        entry,
                        || VersionedFileBytes {
                            value: bytes.map(|x| x.to_owned()),
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

    pub fn repo_files_details_download_temp_file(
        self: Arc<Self>,
        details_id: u32,
        local_base_path: String,
        on_done: Box<dyn TransfersDownloadDone>,
    ) {
        self.clone().spawn(async move {
            let downloadable = Box::new(MobileDownloadable::TempFile {
                base_path: local_base_path.into(),
                on_open: None,
                on_done,
                parent_path: None,
                temp_path: None,
                path: None,
                content_type: None,
            });

            // error is displayed in the details component
            let _ = self
                .vault
                .clone()
                .repo_files_details_download(details_id, downloadable)
                .await;
        })
    }

    pub fn repo_files_details_edit(&self, details_id: u32) {
        self.vault.repo_files_details_edit(details_id);
    }

    pub fn repo_files_details_edit_cancel(self: Arc<Self>, details_id: u32) {
        self.clone().spawn(async move {
            // error is displayed in the details component
            let _ = self
                .vault
                .clone()
                .repo_files_details_edit_cancel(details_id)
                .await;
        });
    }

    pub fn repo_files_details_set_content(&self, details_id: u32, content: Vec<u8>) {
        self.vault
            .repo_files_details_set_content(details_id, content);
    }

    pub fn repo_files_details_save(self: Arc<Self>, details_id: u32) {
        self.clone().spawn(async move {
            // error is displayed in the details component
            let _ = self.vault.clone().repo_files_details_save(details_id).await;
        });
    }

    pub fn repo_files_details_delete(self: Arc<Self>, details_id: u32) {
        self.clone().spawn_result(async move {
            match self.vault.repo_files_details_delete(details_id).await {
                Ok(()) => Ok(()),
                Err(vault_core::repo_files::errors::DeleteFileError::Canceled) => Ok(()),
                Err(err) => Err(err),
            }
        });
    }

    // repo_files_move

    pub fn repo_files_move_is_visible_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::RepoFilesMove, Event::RepoFiles, Event::DirPickers],
            cb,
            self.subscription_data.repo_files_move_is_visible.clone(),
            move |vault| vault.with_state(|state| state.repo_files_move.is_some()),
        )
    }

    pub fn repo_files_move_is_visible_data(&self, id: u32) -> Option<bool> {
        self.get_data(
            id,
            self.subscription_data.repo_files_move_is_visible.clone(),
        )
    }

    pub fn repo_files_move_info_subscribe(&self, cb: Box<dyn SubscriptionCallback>) -> u32 {
        self.subscribe(
            &[Event::RepoFilesMove, Event::RepoFiles, Event::DirPickers],
            cb,
            self.subscription_data.repo_files_move_info.clone(),
            move |vault| {
                vault.with_state(|state| {
                    state
                        .repo_files_move
                        .as_ref()
                        .map(|files_move| RepoFilesMoveInfo {
                            repo_id: files_move.repo_id.0.clone(),
                            src_files_count: files_move.src_paths.len() as u32,
                            mode: (&files_move.mode).into(),
                            encrypted_dest_path_chain: vault_core::utils::path_utils::paths_chain(
                                &files_move.dest_path.0,
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

    pub fn repo_files_move_info_data(&self, id: u32) -> Option<RepoFilesMoveInfo> {
        self.get_data(id, self.subscription_data.repo_files_move_info.clone())
            .flatten()
    }

    pub fn repo_files_move_set_dest_path(&self, encrypted_dest_path: String) {
        self.vault
            .repo_files_move_set_dest_path(EncryptedPath(encrypted_dest_path))
    }

    pub fn repo_files_move_move_files(self: Arc<Self>) {
        self.clone()
            .spawn_result(async move { self.vault.repo_files_move_move_files().await })
    }

    pub fn repo_files_move_cancel(&self) {
        self.vault.repo_files_move_cancel()
    }

    // local_files

    pub fn local_files_file_info(
        &self,
        name: String,
        typ: LocalFileType,
        size: Option<i64>,
        modified: Option<i64>,
    ) -> LocalFile {
        let full_name = name;
        let id = uuid::Uuid::new_v4().to_string();
        let name = vault_core::utils::path_utils::path_to_name(&full_name).unwrap_or(&full_name);
        let name_lower = name.to_lowercase();
        let (ext, size_display, modified, category) = match &typ {
            LocalFileType::Dir => (None, "".into(), None, FileCategory::Folder),
            LocalFileType::File => {
                let ext =
                    vault_core::utils::name_utils::name_to_ext(&name_lower).map(str::to_string);
                let category = ext
                    .as_ref()
                    .and_then(|ext| vault_core::files::file_category::ext_to_file_category(ext))
                    .as_ref()
                    .map(Into::into)
                    .unwrap_or(FileCategory::Generic);
                let size_display = size
                    .map(|size| vault_core::files::file_size::size_display(size))
                    .unwrap_or("".into());

                (ext, size_display, modified, category)
            }
        };
        let file_icon_attrs = FileIconAttrs {
            category: category.clone(),
            is_dl: false,
            is_ul: false,
            is_download_transfer: false,
            is_upload_transfer: false,
            is_export: false,
            is_import: false,
            is_android: false,
            is_ios: false,
            is_vault_repo: false,
            is_error: false,
        };

        LocalFile {
            id,
            name: full_name,
            ext,
            typ,
            size_display,
            modified,
            category,
            file_icon_attrs,
        }
    }

    // version

    pub fn version(&self) -> Version {
        self.version.clone().into()
    }
}

// FakeRemote

#[derive(Error, Debug, Clone, PartialEq)]
pub enum FakeRemoteError {
    #[error("fake remote error: {reason}")]
    Err { reason: String },
    #[error("not implemented")]
    NotImplemented,
}

impl From<uniffi::UnexpectedUniFFICallbackError> for FakeRemoteError {
    fn from(err: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::Err { reason: err.reason }
    }
}

#[cfg(debug_assertions)]
impl From<vault_fake_remote::fake_remote::errors::FakeRemoteServerStartError> for FakeRemoteError {
    fn from(err: vault_fake_remote::fake_remote::errors::FakeRemoteServerStartError) -> Self {
        Self::Err {
            reason: err.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FakeRemoteStarted {
    pub http_url: String,
    pub https_url: String,
}

pub struct FakeRemote {
    #[cfg(debug_assertions)]
    pub fake_remote_app: vault_fake_remote::fake_remote::app::FakeRemoteApp,
}

#[cfg(debug_assertions)]
impl FakeRemote {
    pub fn new(http_addr: String, https_addr: String) -> Result<Self, FakeRemoteError> {
        use vault_fake_remote::fake_remote::files::objects::memory_object_provider::MemoryObjectProvider;

        mobile_logger::try_init_env_logger();

        let config = vault_fake_remote::fake_remote::app::FakeRemoteAppConfig {
            http_addr: http_addr.parse().unwrap(),
            https_addr: https_addr.parse().unwrap(),
            object_provider: Arc::new(Box::new(MemoryObjectProvider::new())),
            user_id: "b2977f16-4766-4528-a26f-4b0b13bf2c9c".into(),
            mount_id: "9fd62581-3bad-478a-702b-01937d2bf7f1".into(),
            oauth2_access_token: "f1fed68a-6b5c-4067-928e-40ed48dd2589".into(),
            oauth2_refresh_token: "a126768a-ce0b-4b93-8a9b-809f02f4c000".into(),
            create_vault_repo: false,
        };

        let fake_remote_app = RT.block_on(vault_fake_remote::fake_remote::app::FakeRemoteApp::new(
            config,
            RT.clone(),
        ));

        Ok(Self { fake_remote_app })
    }

    pub fn start(&self) -> Result<FakeRemoteStarted, FakeRemoteError> {
        let (http_url, https_url) = RT.block_on(self.fake_remote_app.start())?;

        Ok(FakeRemoteStarted {
            http_url,
            https_url,
        })
    }

    pub fn stop(&self) -> Result<(), FakeRemoteError> {
        RT.block_on(self.fake_remote_app.stop());

        Ok(())
    }
}

#[cfg(not(debug_assertions))]
impl FakeRemote {
    pub fn new(_http_addr: String, _https_addr: String) -> Result<Self, FakeRemoteError> {
        return Err(FakeRemoteError::NotImplemented);
    }

    pub fn start(&self) -> Result<FakeRemoteStarted, FakeRemoteError> {
        return Err(FakeRemoteError::NotImplemented);
    }

    pub fn stop(&self) -> Result<(), FakeRemoteError> {
        return Err(FakeRemoteError::NotImplemented);
    }
}

uniffi::include_scaffolding!("vault-mobile");
