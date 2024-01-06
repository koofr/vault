use std::{fs::Metadata, path::PathBuf};

use async_trait::async_trait;
use tokio::fs::File;
use tokio_util::compat::TokioAsyncReadCompatExt;

use vault_core::{
    common::state::{BoxAsyncRead, SizeInfo},
    transfers::{errors::UploadableError, uploadable::Uploadable},
};

pub struct FileUploadable {
    pub path: PathBuf,
    pub cleanup: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
}

#[async_trait]
impl Uploadable for FileUploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError> {
        Ok(SizeInfo::Exact(file_size(
            &File::open(&self.path).await?.metadata().await?,
        )))
    }

    async fn is_retriable(&self) -> Result<bool, UploadableError> {
        Ok(true)
    }

    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError> {
        let file = File::open(&self.path).await?;
        let size = SizeInfo::Exact(file_size(&file.metadata().await?));

        Ok((Box::pin(file.compat()), size))
    }
}

impl Drop for FileUploadable {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

#[cfg(target_os = "windows")]
pub fn file_size(metadata: &Metadata) -> i64 {
    use std::os::windows::fs::MetadataExt;
    metadata.file_size() as i64
}

#[cfg(not(target_os = "windows"))]
pub fn file_size(metadata: &Metadata) -> i64 {
    use std::os::unix::fs::MetadataExt;
    metadata.size() as i64
}
