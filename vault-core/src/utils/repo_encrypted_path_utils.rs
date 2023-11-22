use crate::{
    common::errors::InvalidPathError,
    types::{EncryptedName, EncryptedPath},
};

use super::path_utils;

pub fn path_to_name(path: &EncryptedPath) -> Option<EncryptedName> {
    path_utils::path_to_name(&path.0).map(|x| EncryptedName(x.to_owned()))
}

pub fn join_path_name(path: &EncryptedPath, name: &EncryptedName) -> EncryptedPath {
    EncryptedPath(path_utils::join_path_name(&path.0, &name.0))
}

pub fn join_paths(parent_path: &EncryptedPath, path: &EncryptedPath) -> EncryptedPath {
    EncryptedPath(path_utils::join_paths(&parent_path.0, &path.0))
}

pub fn parent_path(path: &EncryptedPath) -> Option<EncryptedPath> {
    path_utils::parent_path(&path.0).map(|x| EncryptedPath(x.to_owned()))
}

pub fn split_parent_name(path: &EncryptedPath) -> Option<(EncryptedPath, EncryptedName)> {
    path_utils::split_parent_name(&path.0).map(|(parent, name)| {
        (
            EncryptedPath(parent.to_owned()),
            EncryptedName(name.to_owned()),
        )
    })
}

pub fn paths_chain(path: &EncryptedPath) -> Vec<EncryptedPath> {
    path_utils::paths_chain(&path.0)
        .into_iter()
        .map(EncryptedPath)
        .collect()
}

pub fn normalize_path(path: &EncryptedPath) -> Result<EncryptedPath, InvalidPathError> {
    path_utils::normalize_path(&path.0).map(EncryptedPath)
}
