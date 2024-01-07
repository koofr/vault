use std::path::PathBuf;

use async_trait::async_trait;
use futures::future::BoxFuture;
use tokio::fs;

use tokio_util::compat::TokioAsyncReadCompatExt;
use vault_core::{
    common::state::{BoxAsyncWrite, SizeInfo},
    transfers::{
        downloadable::{Downloadable, DownloadableStatus},
        errors::DownloadableError,
    },
};

use crate::file_utils;

pub struct PickFileDownloadable {
    pub pick_file: Box<
        dyn Fn(String) -> BoxFuture<'static, Result<PathBuf, DownloadableError>>
            + Send
            + Sync
            + 'static,
    >,
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

    pub path: Option<PathBuf>,
    pub content_type: Option<String>,
}

#[async_trait]
impl Downloadable for PickFileDownloadable {
    async fn is_retriable(&self) -> Result<bool, DownloadableError> {
        Ok(true)
    }

    async fn is_openable(&self) -> Result<bool, DownloadableError> {
        Ok(self.on_open.is_some())
    }

    async fn exists(
        &mut self,
        _name: String,
        _unique_name: String,
    ) -> Result<bool, DownloadableError> {
        Ok(false)
    }

    async fn writer(
        &mut self,
        name: String,
        _size: SizeInfo,
        content_type: Option<String>,
        _unique_name: Option<String>,
    ) -> Result<(BoxAsyncWrite, String), DownloadableError> {
        let name = file_utils::cleanup_name(&name);

        let path_buf = match self.path.as_ref() {
            Some(path) => path.clone(),
            None => (self.pick_file)(name).await?,
        };

        let name = path_buf
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .ok_or_else(|| {
                DownloadableError::from(std::io::Error::from(std::io::ErrorKind::InvalidInput))
            })?;

        let file = fs::File::create(&path_buf).await?;

        self.path = Some(path_buf);

        self.content_type = content_type;

        Ok((Box::pin(file.compat()), name))
    }

    async fn done(
        &self,
        res: Result<DownloadableStatus, DownloadableError>,
    ) -> Result<(), DownloadableError> {
        if res.is_ok() {
            if let Some(path) = self.path.as_ref() {
                (self.on_done)(path.to_owned(), self.content_type.clone())?;
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
