use crate::types::{EncryptedPath, RepoId};

#[derive(Debug, Clone)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveInfo {
    pub repo_id: RepoId,
    pub src_paths: Vec<EncryptedPath>,
    pub mode: RepoFilesMoveMode,
    pub dir_picker_id: u32,
    pub dest_path: EncryptedPath,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveState {
    pub repo_id: RepoId,
    pub src_paths: Vec<EncryptedPath>,
    pub mode: RepoFilesMoveMode,
    pub dest_path: EncryptedPath,
    pub dir_picker_id: u32,
}
