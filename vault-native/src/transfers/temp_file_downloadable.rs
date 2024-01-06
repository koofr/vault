use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::fs;
use tokio_util::compat::TokioAsyncReadCompatExt;
use uuid::Uuid;

use vault_core::{
    common::state::{BoxAsyncWrite, SizeInfo},
    transfers::{
        downloadable::{Downloadable, DownloadableStatus},
        errors::DownloadableError,
    },
};

use crate::file_utils;

pub struct TempFileDownloadable {
    pub base_path: PathBuf,
    pub on_open: Option<
        Box<
            dyn Fn(PathBuf, Option<String>) -> Result<(), DownloadableError>
                + Send
                + Sync
                + 'static,
        >,
    >,
    pub on_done: Box<
        dyn Fn(PathBuf, Option<String>) -> Result<(), DownloadableError> + Send + Sync + 'static,
    >,
    pub parent_path: Option<PathBuf>,

    /// file is first written to temp_path and then renamed to path
    pub temp_path: Option<PathBuf>,
    pub path: Option<PathBuf>,
    pub content_type: Option<String>,
}

#[async_trait]
impl Downloadable for TempFileDownloadable {
    async fn is_retriable(&self) -> Result<bool, DownloadableError> {
        Ok(true)
    }

    async fn is_openable(&self) -> Result<bool, DownloadableError> {
        Ok(self.on_open.is_some())
    }

    async fn exists(
        &mut self,
        name: String,
        unique_name: String,
    ) -> Result<bool, DownloadableError> {
        let local_parent_path = self.base_path.join(unique_name);
        let local_path = local_parent_path.join(&name);

        let exists = fs::try_exists(&local_path).await?;

        if exists {
            self.parent_path = Some(local_parent_path);
            self.path = Some(local_path);
        }

        Ok(exists)
    }

    async fn writer(
        &mut self,
        name: String,
        _size: SizeInfo,
        content_type: Option<String>,
        unique_name: Option<String>,
    ) -> Result<(BoxAsyncWrite, String), DownloadableError> {
        let name = file_utils::cleanup_name(&name);

        let parent_name = unique_name.clone().unwrap_or(Uuid::new_v4().to_string());

        let local_parent_path = Path::new(&self.base_path).join(parent_name);
        fs::create_dir_all(&local_parent_path).await?;
        self.parent_path = Some(local_parent_path.clone());

        let local_temp_path = local_parent_path.join(&Uuid::new_v4().to_string());
        let file = fs::File::create(&local_temp_path).await?;
        self.temp_path = Some(local_temp_path);

        let local_path = local_parent_path.join(&name);
        self.path = Some(local_path);

        self.content_type = content_type;

        Ok((Box::pin(file.compat()), name))
    }

    async fn done(
        &self,
        res: Result<DownloadableStatus, DownloadableError>,
    ) -> Result<(), DownloadableError> {
        if let Some(parent_path) = self.parent_path.as_ref() {
            if res.is_ok() {
                if let Some(path) = self.path.as_ref() {
                    if let Some(temp_path) = self.temp_path.as_ref() {
                        fs::rename(temp_path, path).await?;
                    }

                    (self.on_done)(path.to_owned(), self.content_type.clone())?;
                }
            } else {
                let _ = fs::remove_dir_all(parent_path);
            }
        }

        Ok(())
    }

    async fn open(&self) -> Result<(), DownloadableError> {
        match (&self.on_open, self.path.as_ref()) {
            (Some(on_open), Some(path)) => Ok(on_open(path.to_owned(), self.content_type.clone())?),
            _ => Err(DownloadableError::NotOpenable),
        }
    }
}
