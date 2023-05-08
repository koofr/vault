use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    pin::Pin,
};

use futures::AsyncRead;

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    file_types::file_category::FileCategory,
    remote::{models, RemoteFileUploadConflictResolution},
    remote_files::state::RemoteFileType,
};

#[derive(Clone)]
pub struct RepoFilesLocation {
    pub repo_id: String,
    pub path: String,
}

#[derive(Clone)]
pub struct RepoFilesBreadcrumb {
    pub id: String,
    pub repo_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoFilePath {
    Decrypted {
        path: String,
    },
    DecryptError {
        parent_path: String,
        encrypted_name: String,
        error: DecryptFilenameError,
    },
}

impl RepoFilePath {
    pub fn decrypted_path<'a>(&'a self) -> Result<&'a str, DecryptFilenameError> {
        match self {
            Self::Decrypted { path } => Ok(path),
            Self::DecryptError { error, .. } => Err(error.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoFileName {
    Decrypted {
        name: String,
        name_lower: String,
    },
    DecryptError {
        encrypted_name: String,
        encrypted_name_lower: String,
        error: DecryptFilenameError,
    },
}

impl RepoFileName {
    pub fn decrypted_name<'a>(&'a self) -> Result<&'a str, DecryptFilenameError> {
        match self {
            Self::Decrypted { name, .. } => Ok(name),
            Self::DecryptError { error, .. } => Err(error.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoFileType {
    Dir,
    File,
}

impl RepoFileType {
    pub fn is_file(&self) -> bool {
        match self {
            Self::Dir => false,
            Self::File => true,
        }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Self::Dir => true,
            Self::File => false,
        }
    }
}

impl Ord for RepoFileType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Dir, Self::Dir) => Ordering::Equal,
            (Self::Dir, Self::File) => Ordering::Less,
            (Self::File, Self::Dir) => Ordering::Greater,
            (Self::File, Self::File) => Ordering::Equal,
        }
    }
}

impl PartialOrd for RepoFileType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&RemoteFileType> for RepoFileType {
    fn from(typ: &RemoteFileType) -> Self {
        match typ {
            RemoteFileType::Dir => Self::Dir,
            RemoteFileType::File => Self::File,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepoFileSize {
    Decrypted {
        size: i64,
    },
    DecryptError {
        encrypted_size: i64,
        error: DecryptSizeError,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoFile {
    pub id: String,
    pub mount_id: String,
    pub remote_path: String,
    pub repo_id: String,
    pub path: RepoFilePath,
    pub name: RepoFileName,
    pub ext: Option<String>,
    pub content_type: Option<String>,
    pub typ: RepoFileType,
    pub size: RepoFileSize,
    pub modified: i64,
    pub remote_hash: Option<String>,
    pub category: FileCategory,
}

impl RepoFile {
    pub fn decrypted_path<'a>(&'a self) -> Result<&'a str, DecryptFilenameError> {
        self.path.decrypted_path()
    }

    pub fn decrypted_name<'a>(&'a self) -> Result<&'a str, DecryptFilenameError> {
        self.name.decrypted_name()
    }

    pub fn name_lower_force<'a>(&'a self) -> &'a str {
        match &self.name {
            RepoFileName::Decrypted { name_lower, .. } => name_lower,
            RepoFileName::DecryptError {
                encrypted_name_lower,
                ..
            } => encrypted_name_lower,
        }
    }

    pub fn size_force(&self) -> i64 {
        match self.size {
            RepoFileSize::Decrypted { size } => size,
            RepoFileSize::DecryptError { encrypted_size, .. } => encrypted_size,
        }
    }
}

#[derive(Clone, Default)]
pub struct RepoFilesState {
    pub files: HashMap<String, RepoFile>,
    pub children: HashMap<String, Vec<String>>,
    pub loaded_roots: HashSet<String>,
}

pub enum RepoFilesUploadConflictResolution {
    Overwrite {
        if_remote_size: Option<i64>,
        if_remote_modified: Option<i64>,
        if_remote_hash: Option<String>,
    },
    Error,
}

impl Into<RemoteFileUploadConflictResolution> for RepoFilesUploadConflictResolution {
    fn into(self) -> RemoteFileUploadConflictResolution {
        match self {
            Self::Overwrite {
                if_remote_size,
                if_remote_modified,
                if_remote_hash,
            } => RemoteFileUploadConflictResolution::Overwrite {
                if_size: if_remote_size,
                if_modified: if_remote_modified,
                if_hash: if_remote_hash,
            },
            Self::Error => RemoteFileUploadConflictResolution::Error,
        }
    }
}

pub trait RepoFileUploadable {
    fn size(&self) -> Option<i64>;
    fn reader(&self) -> Pin<Box<dyn AsyncRead + Send + Sync + 'static>>;
}

pub struct RepoFilesUploadResult {
    pub file_id: String,
    pub name: String,
    pub remote_file: models::FilesFile,
}

#[derive(Clone, PartialEq, Eq)]
pub enum RepoFilesSortField {
    Name,
    Size,
    Modified,
}

impl Default for RepoFilesSortField {
    fn default() -> Self {
        Self::Name
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum RepoFilesSortDirection {
    Asc,
    Desc,
}

impl Default for RepoFilesSortDirection {
    fn default() -> Self {
        Self::Asc
    }
}

impl RepoFilesSortDirection {
    pub fn reverse(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }

    pub fn ordering(&self, ordering: Ordering) -> Ordering {
        match self {
            Self::Asc => ordering,
            Self::Desc => ordering.reverse(),
        }
    }
}

#[derive(Clone, Default)]
pub struct RepoFilesSort {
    pub field: RepoFilesSortField,
    pub direction: RepoFilesSortDirection,
}
