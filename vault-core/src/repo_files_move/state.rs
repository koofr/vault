use crate::types::{DecryptedPath, RepoId};

#[derive(Debug, Clone)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveInfo {
    pub repo_id: RepoId,
    pub src_paths: Vec<DecryptedPath>,
    pub mode: RepoFilesMoveMode,
    pub dir_picker_id: u32,
    pub dest_path: DecryptedPath,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveState {
    pub repo_id: RepoId,
    pub src_paths: Vec<DecryptedPath>,
    pub mode: RepoFilesMoveMode,
    pub dest_path: DecryptedPath,
    pub dir_picker_id: u32,
}
