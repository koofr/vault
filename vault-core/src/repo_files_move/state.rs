#[derive(Debug, Clone)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveInfo {
    pub repo_id: String,
    pub src_paths: Vec<String>,
    pub mode: RepoFilesMoveMode,
    pub dir_picker_id: u32,
    pub dest_path: String,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveState {
    pub repo_id: String,
    pub src_paths: Vec<String>,
    pub mode: RepoFilesMoveMode,
    pub dest_path: String,
    pub dir_picker_id: u32,
}
