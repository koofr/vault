use std::collections::{HashMap, HashSet};

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    files::{file_category::FileCategory, file_icon::FileIconAttrs},
    remote::RemoteFileUploadConflictResolution,
    remote_files::state::{RemoteFile, RemoteFileType},
    sort::state::SortDirection,
};

#[derive(Debug, Clone)]
pub struct RepoFilesBreadcrumb {
    pub id: String,
    pub repo_id: String,
    pub path: String,
    pub name: String,
    pub last: bool,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Dir, Self::Dir) => std::cmp::Ordering::Equal,
            (Self::Dir, Self::File) => std::cmp::Ordering::Less,
            (Self::File, Self::Dir) => std::cmp::Ordering::Greater,
            (Self::File, Self::File) => std::cmp::Ordering::Equal,
        }
    }
}

impl PartialOrd for RepoFileType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
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

#[derive(Debug, Clone, PartialEq)]
pub enum RepoFileSize {
    Decrypted {
        size: i64,
    },
    DecryptError {
        encrypted_size: i64,
        error: DecryptSizeError,
    },
}

impl RepoFileSize {
    pub fn decrypted_size(&self) -> Result<i64, DecryptSizeError> {
        match self {
            Self::Decrypted { size } => Ok(*size),
            Self::DecryptError { error, .. } => Err(error.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    pub size: Option<RepoFileSize>,
    pub modified: Option<i64>,
    pub unique_name: String,
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

    pub fn decrypted_size(&self) -> Result<Option<i64>, DecryptSizeError> {
        match &self.size {
            Some(size) => size.decrypted_size().map(Some),
            None => Ok(None),
        }
    }

    pub fn size_force(&self) -> i64 {
        match self.size {
            Some(RepoFileSize::Decrypted { size }) => size,
            Some(RepoFileSize::DecryptError { encrypted_size, .. }) => encrypted_size,
            None => 0,
        }
    }

    pub fn file_icon_attrs(&self) -> FileIconAttrs {
        FileIconAttrs {
            category: self.category.clone(),
            is_dl: false,
            is_ul: false,
            is_export: false,
            is_import: false,
            is_android: false,
            is_ios: false,
            is_vault_repo: false,
            is_error: match &self.name {
                RepoFileName::Decrypted { .. } => false,
                RepoFileName::DecryptError { .. } => true,
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepoFilesState {
    pub files: HashMap<String, RepoFile>,
    pub children: HashMap<String, Vec<String>>,
    pub loaded_roots: HashSet<String>,
}

impl RepoFilesState {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

#[derive(Debug, Clone, Default)]
pub struct RepoFilesMutationState {
    pub removed_files: Vec<(String, String)>,
    pub moved_files: Vec<(String, String, String)>,
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
                ignore_nonexisting: false,
            },
            Self::Error => RemoteFileUploadConflictResolution::Error,
        }
    }
}

#[derive(Debug)]
pub struct RepoFilesUploadResult {
    pub file_id: String,
    pub name: String,
    pub remote_file: RemoteFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default)]
pub struct RepoFilesSort {
    pub field: RepoFilesSortField,
    pub direction: SortDirection,
}
