use async_trait::async_trait;

use crate::common::state::{BoxAsyncWrite, SizeInfo};

use super::errors::DownloadableError;

#[derive(Debug, Clone)]
pub enum DownloadableStatus {
    Downloaded,
    AlreadyExists,
}

#[async_trait]
pub trait Downloadable {
    async fn is_retriable(&self) -> Result<bool, DownloadableError>;

    async fn is_openable(&self) -> Result<bool, DownloadableError>;

    async fn exists(
        &mut self,
        name: String,
        unique_name: String,
    ) -> Result<bool, DownloadableError>;

    async fn writer(
        &mut self,
        name: String,
        size: SizeInfo,
        content_type: Option<String>,
        unique_name: Option<String>,
    ) -> Result<(BoxAsyncWrite, String), DownloadableError>;

    async fn done(
        &self,
        res: Result<DownloadableStatus, DownloadableError>,
    ) -> Result<(), DownloadableError>;

    async fn open(&self) -> Result<(), DownloadableError>;
}

pub type BoxDownloadable = Box<dyn Downloadable + Send + Sync>;
