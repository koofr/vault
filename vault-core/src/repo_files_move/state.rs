#[derive(Clone)]
pub enum RepoFilesMoveMode {
    Copy,
    Move,
}

#[derive(Clone)]
pub struct RepoFilesMoveState {
    pub repo_id: String,
    pub src_file_ids: Vec<String>,
    pub mode: RepoFilesMoveMode,
    pub dir_picker_id: u32,
}
