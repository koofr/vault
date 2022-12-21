use std::collections::HashMap;

use crate::{
    file_types::file_icon_type::FileIconType, repo_files::selectors as repo_files_selectors,
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
    pub icon_type: FileIconType,
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

#[derive(Clone, PartialEq, Debug)]
pub struct RemainingTime {
    pub days: u32,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
}

impl RemainingTime {
    pub fn from_seconds(total_seconds: f64) -> Self {
        let mut total = total_seconds;

        let days = (total / (24.0 * 3600.0)).floor() as u32;
        total %= 24.0 * 3600.0;

        let hours = (total / 3600.0).floor() as u32;
        total %= 3600.0;

        let minutes = (total / 60.0).floor() as u32;
        total %= 60.0;

        let seconds = total.ceil() as u32;

        RemainingTime {
            days,
            hours,
            minutes,
            seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RemainingTime;

    #[test]
    fn test_remaining_time_from_seconds() {
        let remaining_time = RemainingTime::from_seconds(50.0 * 3600.0 + 45.0 * 60.0 + 30.0 + 0.7);

        assert_eq!(
            remaining_time,
            RemainingTime {
                days: 2,
                hours: 2,
                minutes: 45,
                seconds: 31,
            }
        )
    }
}
