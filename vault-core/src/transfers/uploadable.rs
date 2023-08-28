use async_trait::async_trait;

use crate::common::state::{BoxAsyncRead, SizeInfo};

use super::errors::UploadableError;

#[async_trait]
pub trait Uploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError>;
    async fn is_retriable(&self) -> Result<bool, UploadableError>;
    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError>;
}

pub type BoxUploadable = Box<dyn Uploadable + Send + Sync>;
