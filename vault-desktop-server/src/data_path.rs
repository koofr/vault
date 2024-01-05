use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("failed to get data path: {0}")]
pub struct GetDataPathError(String);

pub fn get_data_path(app_id: &str) -> Result<PathBuf, GetDataPathError> {
    #[cfg(not(windows))]
    let data_dir = {
        let home_dir = dirs_sys_next::home_dir()
            .ok_or_else(|| GetDataPathError(format!("failed to get home dir")))?;

        home_dir.join(format!(".{}", app_id))
    };
    #[cfg(windows)]
    let data_dir = {
        let project_dirs = directories_next::ProjectDirs::from("", "", app_id)
            .ok_or_else(|| GetDataPathError(format!("failed to find project dirs for app")))?;

        project_dirs.data_dir().to_path_buf()
    };

    std::fs::create_dir_all(&data_dir)
        .map_err(|err| GetDataPathError(format!("failed to ensure data dir: {}", err)))?;

    Ok(data_dir)
}
