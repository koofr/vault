use crate::{
    common::errors::InvalidPathError,
    types::{DecryptedName, DecryptedPath},
};

use super::path_utils;

pub fn path_to_name(path: &DecryptedPath) -> Option<DecryptedName> {
    path_utils::path_to_name(&path.0).map(|x| DecryptedName(x.to_owned()))
}

pub fn join_path_name(path: &DecryptedPath, name: &DecryptedName) -> DecryptedPath {
    DecryptedPath(path_utils::join_path_name(&path.0, &name.0))
}

pub fn join_paths(parent_path: &DecryptedPath, path: &DecryptedPath) -> DecryptedPath {
    DecryptedPath(path_utils::join_paths(&parent_path.0, &path.0))
}

pub fn parent_path(path: &DecryptedPath) -> Option<DecryptedPath> {
    path_utils::parent_path(&path.0).map(|x| DecryptedPath(x.to_owned()))
}

pub fn split_parent_name(path: &DecryptedPath) -> Option<(DecryptedPath, DecryptedName)> {
    path_utils::split_parent_name(&path.0).map(|(parent, name)| {
        (
            DecryptedPath(parent.to_owned()),
            DecryptedName(name.to_owned()),
        )
    })
}

pub fn paths_chain(path: &DecryptedPath) -> Vec<DecryptedPath> {
    path_utils::paths_chain(&path.0)
        .into_iter()
        .map(DecryptedPath)
        .collect()
}

pub fn normalize_path(path: &DecryptedPath) -> Result<DecryptedPath, InvalidPathError> {
    path_utils::normalize_path(&path.0).map(DecryptedPath)
}
