use std::{fmt::Debug, io::SeekFrom, ops::RangeInclusive, path::PathBuf, pin::Pin};

use async_trait::async_trait;
use futures::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::io::AsyncSeekExt;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use vault_core::utils::md5_reader;

use super::object_provider::{ObjectProvider, ObjectProviderError};

#[derive(Debug)]
pub struct FsObjectProvider {
    data_path: PathBuf,
}

impl FsObjectProvider {
    pub fn new(data_path: PathBuf) -> Self {
        Self { data_path }
    }

    fn get_object_path(&self, object_id: &str) -> PathBuf {
        self.data_path.join(&object_id)
    }
}

#[async_trait]
impl ObjectProvider for FsObjectProvider {
    async fn get(
        &self,
        object_id: String,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<Pin<Box<dyn AsyncRead + Send + Sync + 'static>>, ObjectProviderError> {
        let path = self.get_object_path(&object_id);

        let mut file = tokio::fs::File::open(path).await?;

        match range {
            Some(range) => {
                file.seek(SeekFrom::Start(*range.start())).await?;

                Ok(Box::pin(
                    file.compat().take(range.end() - range.start() + 1),
                ))
            }
            None => Ok(Box::pin(file.compat())),
        }
    }

    async fn put(
        &self,
        object_id: String,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
    ) -> Result<(u64, String), ObjectProviderError> {
        let path = self.get_object_path(&object_id);

        let mut md5_reader = md5_reader::MD5Reader::new(reader);

        let mut writer = tokio::fs::File::create(path).await?.compat_write();

        let size = futures::io::copy(&mut md5_reader, &mut writer).await?;

        writer.flush().await?;

        let hash = md5_reader.hex_digest();

        Ok((size, hash))
    }

    async fn delete(&self, object_id: String) -> Result<(), ObjectProviderError> {
        let path = self.get_object_path(&object_id);

        Ok(tokio::fs::remove_file(&path).await?)
    }
}
