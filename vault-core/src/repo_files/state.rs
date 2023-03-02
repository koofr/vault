use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    pin::Pin,
};

use futures::AsyncRead;

use crate::{
    cipher::errors::{DecryptFilenameError, DecryptSizeError},
    file_types::file_icon_type::FileIconType,
    remote::RemoteFileUploadConflictResolution,
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

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone, PartialEq, Eq)]
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

#[derive(Clone)]
pub enum RepoFileSize {
    Decrypted {
        size: i64,
    },
    DecryptError {
        encrypted_size: i64,
        error: DecryptSizeError,
    },
}

#[derive(Clone)]
pub struct RepoFile {
    pub id: String,
    pub remote_file_id: String,
    pub repo_id: String,
    pub path: RepoFilePath,
    pub name: RepoFileName,
    pub typ: RepoFileType,
    pub size: RepoFileSize,
    pub modified: i64,
    pub icon_type: FileIconType,
}

impl RepoFile {
    pub fn decrypted_path<'a>(&'a self) -> Result<&'a str, DecryptFilenameError> {
        self.path.decrypted_path()
    }

    pub fn decrypted_name<'a>(&'a self) -> Result<&'a str, DecryptFilenameError> {
        self.name.decrypted_name()
    }
}

#[derive(Clone, Default)]
pub struct RepoFilesState {
    pub files: HashMap<String, RepoFile>,
    pub children: HashMap<String, Vec<String>>,
    pub loaded_roots: HashSet<String>,
}

pub struct RepoFileReader {
    pub name: String,
    pub size: i64,
    pub reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
}

pub enum RepoFilesUploadConflictResolution {
    Overwrite,
    Error,
}

impl Into<RemoteFileUploadConflictResolution> for RepoFilesUploadConflictResolution {
    fn into(self) -> RemoteFileUploadConflictResolution {
        match self {
            Self::Overwrite => RemoteFileUploadConflictResolution::Overwrite,
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
}
