use crate::{
    common::errors::InvalidPathError,
    types::{RemoteName, RemoteNameLower, RemotePath, RemotePathLower},
};

use super::path_utils;

pub fn path_to_name(path: &RemotePath) -> Option<RemoteName> {
    path_utils::path_to_name(&path.0).map(|x| RemoteName(x.to_owned()))
}

pub fn join_path_name(path: &RemotePath, name: &RemoteName) -> RemotePath {
    RemotePath(path_utils::join_path_name(&path.0, &name.0))
}

pub fn join_path_name_lower(path: &RemotePathLower, name: &RemoteNameLower) -> RemotePathLower {
    RemotePathLower(path_utils::join_path_name(&path.0, &name.0))
}

pub fn join_paths(parent_path: &RemotePath, path: &RemotePath) -> RemotePath {
    RemotePath(path_utils::join_paths(&parent_path.0, &path.0))
}

pub fn parent_path(path: &RemotePath) -> Option<RemotePath> {
    path_utils::parent_path(&path.0).map(|x| RemotePath(x.to_owned()))
}

pub fn split_parent_name(path: &RemotePath) -> Option<(RemotePath, RemoteName)> {
    path_utils::split_parent_name(&path.0)
        .map(|(parent, name)| (RemotePath(parent.to_owned()), RemoteName(name.to_owned())))
}

pub fn paths_chain(path: &RemotePath) -> Vec<RemotePath> {
    path_utils::paths_chain(&path.0)
        .into_iter()
        .map(RemotePath)
        .collect()
}

pub fn normalize_path(path: &RemotePath) -> Result<RemotePath, InvalidPathError> {
    path_utils::normalize_path(&path.0).map(RemotePath)
}
