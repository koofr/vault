use std::collections::HashMap;

use crate::{
    file_types::file_category::FileCategory, repo_files::selectors as repo_files_selectors,
};

use super::errors::UploadError;

#[derive(Clone)]
pub enum FileUploadState {
    Waiting,
    Uploading,
    Failed { error: UploadError },
    Done,
}

#[derive(Clone)]
pub struct FileUpload {
    pub id: u32,
    pub repo_id: String,
    pub parent_path: String,
    pub name: String,
    pub autorename_name: Option<String>,
    pub size: Option<i64>,
    pub category: FileCategory,
    pub started: i64,
    pub is_persistent: bool,
    pub state: FileUploadState,
    pub uploaded_bytes: i64,
    pub attempts: u32,
    pub order: u32,
}

impl FileUpload {
    pub fn parent_id(&self) -> String {
        repo_files_selectors::get_file_id(&self.repo_id, &self.parent_path)
    }
}

#[derive(Clone, Default)]
pub struct UploadsState {
    pub files: HashMap<u32, FileUpload>,
    pub next_id: u32,
    pub started: Option<i64>,
    pub uploading_count: u32,
    pub done_count: u32,
    pub failed_count: u32,
    pub total_count: u32,
    pub done_bytes: i64,
    pub failed_bytes: i64,
    pub total_bytes: i64,
}
