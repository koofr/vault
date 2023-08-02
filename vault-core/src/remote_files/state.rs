use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use crate::{
    common::state::BoxAsyncRead,
    dir_pickers::state::DirPickerItemType,
    files::{file_category::FileCategory, file_icon::FileIconAttrs},
    remote::models,
    sort::state::SortDirection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountOrigin {
    Hosted,
    Desktop,
    Dropbox,
    Googledrive,
    Onedrive,
    Share,
    Other { origin: String },
}

impl MountOrigin {
    pub fn order(&self) -> u32 {
        match self {
            Self::Hosted => 0,
            Self::Desktop => 1,
            Self::Dropbox => 2,
            Self::Googledrive => 3,
            Self::Onedrive => 4,
            Self::Share => 5,
            Self::Other { origin: _ } => 6,
        }
    }
}

impl From<&str> for MountOrigin {
    fn from(origin: &str) -> Self {
        match origin {
            "hosted" => Self::Hosted,
            "desktop" => Self::Desktop,
            "dropbox" => Self::Dropbox,
            "googledrive" => Self::Googledrive,
            "onedrive" => Self::Onedrive,
            "share" => Self::Share,
            _ => Self::Other {
                origin: origin.to_owned(),
            },
        }
    }
}

// TODO remove, map directly from mount to DirPickerItemType
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFileExtType {
    Import,
    Export,
    Hosted,
    Desktop,
    DesktopOffline,
    Dropbox,
    Googledrive,
    Onedrive,
}

impl From<&Mount> for RemoteFileExtType {
    fn from(mount: &Mount) -> Self {
        match &mount.typ {
            MountType::Device => match &mount.origin {
                MountOrigin::Hosted => RemoteFileExtType::Hosted,
                MountOrigin::Desktop => {
                    if mount.online {
                        RemoteFileExtType::Desktop
                    } else {
                        RemoteFileExtType::DesktopOffline
                    }
                }
                MountOrigin::Dropbox => RemoteFileExtType::Dropbox,
                MountOrigin::Googledrive => RemoteFileExtType::Googledrive,
                MountOrigin::Onedrive => RemoteFileExtType::Onedrive,
                MountOrigin::Share => RemoteFileExtType::Hosted,
                MountOrigin::Other { origin: _ } => RemoteFileExtType::Hosted,
            },
            MountType::Import => RemoteFileExtType::Import,
            MountType::Export => RemoteFileExtType::Export,
        }
    }
}

impl Into<DirPickerItemType> for RemoteFileExtType {
    fn into(self) -> DirPickerItemType {
        match self {
            Self::Import => DirPickerItemType::Import,
            Self::Export => DirPickerItemType::Export,
            Self::Hosted => DirPickerItemType::Hosted,
            Self::Desktop => DirPickerItemType::Desktop,
            Self::DesktopOffline => DirPickerItemType::DesktopOffline,
            Self::Dropbox => DirPickerItemType::Dropbox,
            Self::Googledrive => DirPickerItemType::Googledrive,
            Self::Onedrive => DirPickerItemType::Onedrive,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MountType {
    Device,
    Export,
    Import,
}

impl From<&str> for MountType {
    fn from(typ: &str) -> Self {
        match typ {
            "device" => Self::Device,
            "export" => Self::Export,
            "import" => Self::Import,
            _ => Self::Device,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mount {
    pub id: String,
    pub name: String,
    pub name_lower: String,
    pub typ: MountType,
    pub origin: MountOrigin,
    pub online: bool,
    pub is_primary: bool,
}

impl From<models::Mount> for Mount {
    fn from(mount: models::Mount) -> Self {
        let name_lower = mount.name.to_lowercase();

        Self {
            id: mount.id,
            name: mount.name,
            name_lower,
            typ: mount.typ.as_str().into(),
            origin: mount.origin.as_str().into(),
            online: mount.online,
            is_primary: mount.is_primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFilesLocation {
    pub mount_id: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct RemoteFilesBreadcrumb {
    pub id: String,
    pub mount_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFileType {
    Dir,
    File,
}

impl Ord for RemoteFileType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Dir, Self::Dir) => Ordering::Equal,
            (Self::Dir, Self::File) => Ordering::Less,
            (Self::File, Self::Dir) => Ordering::Greater,
            (Self::File, Self::File) => Ordering::Equal,
        }
    }
}

impl PartialOrd for RemoteFileType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&str> for RemoteFileType {
    fn from(typ: &str) -> Self {
        match typ {
            "dir" => Self::Dir,
            "file" => Self::File,
            _ => Self::File,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub id: String,
    pub mount_id: String,
    pub path: String,
    pub name: String,
    pub name_lower: String,
    pub ext: Option<String>,
    pub typ: RemoteFileType,
    pub size: Option<i64>,
    pub modified: Option<i64>,
    pub hash: Option<String>,
    pub unique_id: String,
    pub category: FileCategory,
}

impl RemoteFile {
    pub fn get_location(&self) -> RemoteFilesLocation {
        RemoteFilesLocation {
            mount_id: self.mount_id.clone(),
            path: self.path.clone(),
        }
    }

    pub fn file_icon_attrs(&self, mount: Option<&Mount>) -> FileIconAttrs {
        FileIconAttrs {
            category: self.category.clone(),
            is_dl: false,
            is_ul: false,
            is_export: mount
                .map(|mount| matches!(mount.typ, MountType::Export))
                .unwrap_or(false),
            is_import: mount
                .map(|mount| matches!(mount.typ, MountType::Import))
                .unwrap_or(false),
            is_android: false, // TODO we need to know if file's mount is primary and check path_lower, put it in RemoteFile directly (is_android_media)
            is_ios: false,     // TODO
            is_vault_repo: false, // TODO put in RemoteFile
            is_error: false,
        }
    }
}

pub struct RemoteFilesFileReader {
    pub file: RemoteFile,
    pub size: i64,
    pub reader: BoxAsyncRead,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFilesState {
    pub mounts: HashMap<String, Mount>,
    pub places_loaded: bool,
    pub place_mount_ids: Vec<String>,
    pub online_place_mount_ids: Vec<String>,
    pub files: HashMap<String, RemoteFile>,
    pub children: HashMap<String, Vec<String>>,
    pub loaded_roots: HashSet<String>,
    pub bookmarks_loaded: bool,
    pub bookmark_file_ids: Vec<String>,
    pub shared_files_loaded: bool,
    pub shared_file_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFilesMutationState {
    pub loaded_roots: Vec<(String, String)>,
    pub created_files: Vec<(String, String)>,
    pub removed_files: Vec<(String, String)>,
    pub moved_files: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFilesSortField {
    Name,
    Size,
    Modified,
}

impl Default for RemoteFilesSortField {
    fn default() -> Self {
        Self::Name
    }
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFilesSort {
    pub field: RemoteFilesSortField,
    pub direction: SortDirection,
}
