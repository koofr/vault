#[derive(Debug, Clone)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

#[derive(Debug, Clone)]
pub struct RepoFilesMoveState {
    pub repo_id: String,
    pub src_file_ids: Vec<String>,
    pub mode: RepoFilesMoveMode,
    pub dir_picker_id: u32,
}
