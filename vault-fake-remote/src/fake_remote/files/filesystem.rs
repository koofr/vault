use std::collections::HashMap;

use http::StatusCode;
use vault_core::{
    remote::{models, remote::RemoteFileTagsSetConditions},
    remote_files_tags::set_tags::set_tags,
    types::RemoteName,
    utils::name_utils,
};

use crate::fake_remote::errors::{ApiErrorCode, FakeRemoteError};

use super::{path::NormalizedPath, Name, Path};

#[derive(Debug)]
pub enum CreateFileConflictResolution {
    Autorename,
    Overwrite {
        if_modified: Option<i64>,
        if_size: Option<i64>,
        if_hash: Option<String>,
        ignore_nonexistent: bool,
    },
    Error,
}

impl CreateFileConflictResolution {
    pub fn parse(
        autorename: Option<String>,
        overwrite: Option<String>,
        overwrite_if_modified: Option<i64>,
        overwrite_if_size: Option<i64>,
        overwrite_if_hash: Option<String>,
        overwrite_ignore_nonexisting: Option<String>,
        overwrite_ignore_nonexistent: Option<String>,
    ) -> Self {
        let overwrite_ignore_nonexistent = overwrite_ignore_nonexistent == Some("".into())
            || overwrite_ignore_nonexisting == Some("".into());

        let overwrite = overwrite == Some("true".into())
            || overwrite_if_modified.is_some()
            || overwrite_if_size.is_some()
            || overwrite_if_hash.is_some()
            || overwrite_ignore_nonexistent;

        let autorename = (autorename.is_none() || autorename != Some("false".into())) && !overwrite;

        if autorename {
            Self::Autorename
        } else if overwrite {
            Self::Overwrite {
                if_modified: overwrite_if_modified,
                if_size: overwrite_if_size,
                if_hash: overwrite_if_hash,
                ignore_nonexistent: overwrite_ignore_nonexistent,
            }
        } else {
            Self::Error
        }
    }
}

#[derive(Debug)]
pub struct MoveFileConditions {
    pub if_modified: Option<i64>,
    pub if_size: Option<i64>,
    pub if_hash: Option<String>,
}

pub struct FilesTagsSetConditions {
    pub if_size: Option<i64>,
    pub if_modified: Option<i64>,
    pub if_hash: Option<String>,
    pub if_old_tags: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct FilesystemFile {
    pub file: models::FilesFile,
    // children paths
    pub children: Vec<NormalizedPath>,
    pub object_id: Option<String>,
}

#[derive(Debug)]
pub struct FilesystemObject {
    pub refs: usize,
}

#[derive(Debug)]
pub struct Filesystem {
    // paths to files
    pub files: HashMap<NormalizedPath, FilesystemFile>,
    // object ids to objects
    pub objects: HashMap<String, FilesystemObject>,
}

impl Filesystem {
    pub fn get_file(&self, path: &NormalizedPath) -> Result<&FilesystemFile, FakeRemoteError> {
        self.files.get(&path).ok_or(FakeRemoteError::ApiError(
            StatusCode::NOT_FOUND,
            ApiErrorCode::NotFound,
            "File not found".into(),
            None,
        ))
    }

    pub fn get_file_mut(
        &mut self,
        path: &NormalizedPath,
    ) -> Result<&mut FilesystemFile, FakeRemoteError> {
        self.files.get_mut(&path).ok_or(FakeRemoteError::ApiError(
            StatusCode::NOT_FOUND,
            ApiErrorCode::NotFound,
            "File not found".into(),
            None,
        ))
    }

    pub fn get_children(&self, file: &FilesystemFile) -> Vec<&FilesystemFile> {
        file.children
            .iter()
            .filter_map(|path| self.get_file(path).ok())
            .collect()
    }

    pub fn create_file_check(
        &self,
        parent_path: &Path,
        name: &Name,
        conflict_resolution: &CreateFileConflictResolution,
    ) -> Result<(), FakeRemoteError> {
        let full_path = parent_path.join_name(name);

        let current_file = self.files.get(&full_path.normalize());

        match conflict_resolution {
            CreateFileConflictResolution::Autorename => {}
            CreateFileConflictResolution::Overwrite {
                if_modified,
                if_size,
                if_hash,
                ignore_nonexistent,
            } => match current_file {
                Some(current_file) => {
                    if let Some(if_modified) = if_modified {
                        if if_modified != &current_file.file.modified {
                            return Err(FakeRemoteError::ApiError(
                                StatusCode::CONFLICT,
                                ApiErrorCode::Conflict,
                                "Overwrite if modified does not match".into(),
                                None,
                            ));
                        }
                    }
                    if let Some(if_size) = if_size {
                        if if_size != &current_file.file.size {
                            return Err(FakeRemoteError::ApiError(
                                StatusCode::CONFLICT,
                                ApiErrorCode::Conflict,
                                "Overwrite if size does not match".into(),
                                None,
                            ));
                        }
                    }
                    if let Some(if_hash) = if_hash {
                        if Some(if_hash) != current_file.file.hash.as_ref() {
                            return Err(FakeRemoteError::ApiError(
                                StatusCode::CONFLICT,
                                ApiErrorCode::Conflict,
                                "Overwrite if hash does not match".into(),
                                None,
                            ));
                        }
                    }
                    if current_file.file.typ != "file" {
                        return Err(FakeRemoteError::ApiError(
                            StatusCode::BAD_REQUEST,
                            ApiErrorCode::NotFile,
                            "Not a file".into(),
                            None,
                        ));
                    }
                }
                None => {
                    if (if_modified.is_some() || if_size.is_some() || if_hash.is_some())
                        && !ignore_nonexistent
                    {
                        return Err(FakeRemoteError::ApiError(
                            StatusCode::CONFLICT,
                            ApiErrorCode::Conflict,
                            "Overwrite file not found".into(),
                            None,
                        ));
                    }
                }
            },
            CreateFileConflictResolution::Error => {
                if current_file.is_some() {
                    return Err(FakeRemoteError::ApiError(
                        // TODO should be BAD_REQUEST for api v2.0
                        StatusCode::CONFLICT,
                        ApiErrorCode::Conflict,
                        "File already exists".into(),
                        None,
                    ));
                }
            }
        }

        let _ = self.get_file(&parent_path.normalize())?;

        Ok(())
    }

    pub fn create_file(
        &mut self,
        parent_path: &Path,
        name: Name,
        modified: i64,
        size: i64,
        hash: String,
        new_tags: HashMap<String, Vec<String>>,
        conflict_resolution: &CreateFileConflictResolution,
        object_id: String,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        self.create_file_check(parent_path, &name, &conflict_resolution)?;

        let name = match conflict_resolution {
            CreateFileConflictResolution::Autorename => {
                Name(name_utils::unused_name(&name.0, |name| {
                    self.files
                        .contains_key(&parent_path.join_name(&Name(name.to_owned())).normalize())
                }))
            }
            CreateFileConflictResolution::Overwrite { .. } => name,
            CreateFileConflictResolution::Error => name,
        };

        let full_path = parent_path.join_name(&name);

        let existing_file = match conflict_resolution {
            CreateFileConflictResolution::Overwrite { .. } => {
                if self.files.contains_key(&full_path.normalize()) {
                    Some(self.delete_file(&full_path, false)?)
                } else {
                    None
                }
            }
            _ => None,
        };

        let ext = name.ext();
        let content_type = ext
            .and_then(|ext| {
                vault_core::files::content_type::ext_to_content_type(&ext).map(str::to_string)
            })
            .unwrap_or_else(|| "application/octet-stream".into());

        let mut tags = existing_file
            .map(|file| file.tags.clone())
            .unwrap_or_else(Default::default);

        for (key, mut new_values) in new_tags {
            match tags.get_mut(&key) {
                Some(values) => {
                    values.append(&mut new_values);
                }
                None => {
                    tags.insert(key, new_values);
                }
            }
        }

        let parent_file = self.get_file_mut(&parent_path.normalize())?;

        parent_file.children.push(full_path.normalize());

        let file = models::FilesFile {
            name: RemoteName(name.0),
            typ: "file".into(),
            modified,
            size,
            content_type,
            hash: Some(hash),
            tags,
        };

        self.files.insert(
            full_path.normalize(),
            FilesystemFile {
                file: file.clone(),
                children: vec![],
                object_id: Some(object_id.clone()),
            },
        );

        self.objects.insert(object_id, FilesystemObject { refs: 1 });

        Ok(file)
    }

    pub fn create_dir(
        &mut self,
        parent_path: &Path,
        name: Name,
        modified: i64,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        let full_path = parent_path.join_name(&name);

        if self.files.contains_key(&full_path.normalize()) {
            return Err(FakeRemoteError::ApiError(
                StatusCode::CONFLICT,
                ApiErrorCode::AlreadyExists,
                "File already exists".into(),
                None,
            ));
        }

        let parent_file = self.get_file_mut(&parent_path.normalize())?;

        parent_file.children.push(full_path.normalize());

        let file = models::FilesFile {
            name: RemoteName(name.0),
            typ: "dir".into(),
            modified,
            size: 0,
            content_type: "".into(),
            hash: None,
            tags: HashMap::new(),
        };

        self.files.insert(
            full_path.normalize(),
            FilesystemFile {
                file: file.clone(),
                children: vec![],
                object_id: None,
            },
        );

        Ok(file)
    }

    pub fn delete_file(
        &mut self,
        path: &Path,
        delete_if_empty: bool,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        if path == &Path::root() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::NotDir,
                "Directory expected".into(),
                None,
            ));
        }

        let path_norm = path.normalize();

        let file = self.get_file(&path_norm)?;

        if delete_if_empty && !file.children.is_empty() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::CONFLICT,
                ApiErrorCode::Conflict,
                "Conflict".into(),
                None,
            ));
        }

        let parent_file = self.get_file_mut(&path.parent().unwrap().normalize())?;

        parent_file
            .children
            .retain(|child_path| child_path != &path_norm);

        let file = self.files.remove(&path_norm).unwrap();

        self.delete_child(&file);

        Ok(file.file)
    }

    fn delete_child(&mut self, file: &FilesystemFile) {
        if let Some(object_id) = &file.object_id {
            if let Some(ref mut object) = self.objects.get_mut(object_id) {
                object.refs -= 1;
            }
        }

        for child_path in file.children.clone() {
            if let Some(file) = self.files.remove(&child_path) {
                self.delete_child(&file);
            }
        }
    }

    pub fn copy_file(
        &mut self,
        path: &Path,
        to_path: Path,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        if path == &Path::root() || path == &to_path || to_path.relative_to(path).is_some() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::CopyIntoSelf,
                "Cannot copy into itself".into(),
                None,
            ));
        }

        if self.get_file(&to_path.normalize()).is_ok() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::CONFLICT,
                ApiErrorCode::AlreadyExists,
                "File already exists".into(),
                None,
            ));
        }

        let file = self.get_file(&path.normalize())?.clone();

        let to_file = models::FilesFile {
            name: RemoteName(to_path.name().unwrap().0),
            ..file.file.clone()
        };

        let to_fs_file = FilesystemFile {
            file: to_file.clone(),
            children: vec![],
            object_id: file.object_id.clone(),
        };

        let to_parent_file = self.get_file_mut(&to_path.parent().unwrap().normalize())?;

        to_parent_file.children.push(to_path.normalize());

        self.copy_path(file, to_path.clone(), to_fs_file);

        Ok(to_file)
    }

    fn copy_path(&mut self, file: FilesystemFile, to_path: Path, to_file: FilesystemFile) {
        let mut to_file = to_file;

        for child_path in file.children.clone() {
            if let Some(child_file) = self.files.get(&child_path) {
                let to_child_path = to_path.join_name(&Name(child_file.file.name.0.clone()));

                let to_child_file = FilesystemFile {
                    children: vec![],
                    ..child_file.clone()
                };

                to_file.children.push(to_child_path.normalize());

                self.copy_path(child_file.clone(), to_child_path, to_child_file);
            }
        }

        if let Some(object_id) = &to_file.object_id {
            if let Some(ref mut object) = self.objects.get_mut(object_id) {
                object.refs += 1;
            }
        }

        self.files.insert(to_path.normalize().clone(), to_file);
    }

    pub fn move_file(
        &mut self,
        path: &Path,
        to_path: Path,
        conditions: &MoveFileConditions,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        if path == &Path::root() || path == &to_path || to_path.relative_to(path).is_some() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::MoveIntoSelf,
                "Cannot move into itself".into(),
                None,
            ));
        }

        if self.get_file(&to_path.normalize()).is_ok() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::CONFLICT,
                ApiErrorCode::AlreadyExists,
                "File already exists".into(),
                None,
            ));
        }

        let file = self.get_file(&path.normalize())?.clone();

        if let Some(if_modified) = conditions.if_modified {
            if &if_modified != &file.file.modified {
                return Err(FakeRemoteError::ApiError(
                    StatusCode::CONFLICT,
                    ApiErrorCode::Conflict,
                    "Move if modified does not match".into(),
                    None,
                ));
            }
        }
        if let Some(if_size) = conditions.if_size {
            if &if_size != &file.file.size {
                return Err(FakeRemoteError::ApiError(
                    StatusCode::CONFLICT,
                    ApiErrorCode::Conflict,
                    "Move if size does not match".into(),
                    None,
                ));
            }
        }
        if let Some(if_hash) = &conditions.if_hash {
            if Some(if_hash) != file.file.hash.as_ref() {
                return Err(FakeRemoteError::ApiError(
                    StatusCode::CONFLICT,
                    ApiErrorCode::Conflict,
                    "Move if hash does not match".into(),
                    None,
                ));
            }
        }

        let to_file = models::FilesFile {
            name: RemoteName(to_path.name().unwrap().0),
            ..file.file.clone()
        };

        let to_fs_file = FilesystemFile {
            file: to_file.clone(),
            children: vec![],
            object_id: file.object_id.clone(),
        };

        let to_parent_file = self.get_file_mut(&to_path.parent().unwrap().normalize())?;

        to_parent_file.children.push(to_path.normalize());

        self.copy_path(file, to_path.clone(), to_fs_file);

        self.delete_file(path, false)?;

        Ok(to_file)
    }

    pub fn tags_set(
        &mut self,
        path: &Path,
        tags: HashMap<String, Vec<String>>,
        conditions: FilesTagsSetConditions,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        let file = self.get_file_mut(&path.normalize())?;

        set_tags(
            Some(file.file.size),
            Some(file.file.modified),
            file.file.hash.as_deref(),
            &mut file.file.tags,
            tags,
            &RemoteFileTagsSetConditions {
                if_size: conditions.if_size,
                if_modified: conditions.if_modified,
                if_hash: conditions.if_hash,
                if_old_tags: conditions.if_old_tags,
            },
        )
        .map_err(|err| {
            FakeRemoteError::ApiError(StatusCode::CONFLICT, ApiErrorCode::Conflict, err, None)
        })?;

        Ok(file.file.clone())
    }
}
