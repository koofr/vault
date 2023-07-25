use std::{os::unix::prelude::MetadataExt, sync::Arc};

use async_trait::async_trait;
use futures::io::Cursor;
use tokio::fs::File;
use tokio_util::compat::TokioAsyncReadCompatExt;

use vault_core::{
    common::state::{BoxAsyncRead, SizeInfo},
    transfers::{errors::UploadableError, uploadable::Uploadable},
};

use crate::{upload_stream_reader::UploadStreamReader, StreamError, UploadStreamProvider};

pub enum MobileUploadable {
    File {
        path: String,
        remove_file_after_upload: bool,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    },
    Stream {
        stream_provider: Arc<Box<dyn UploadStreamProvider>>,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    },
    Bytes {
        bytes: Vec<u8>,
    },
}

#[async_trait]
impl Uploadable for MobileUploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError> {
        match self {
            Self::File { path, .. } => Ok(SizeInfo::Exact(
                File::open(path).await?.metadata().await?.size() as i64,
            )),
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                uploadable_stream_blocking(&tokio_runtime, move || {
                    Ok(stream_provider.size()?.into())
                })
                .await
            }
            Self::Bytes { bytes } => Ok(SizeInfo::Exact(bytes.len() as i64)),
        }
    }

    async fn is_retriable(&self) -> Result<bool, UploadableError> {
        match self {
            Self::File { .. } => Ok(true),
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                uploadable_stream_blocking(&tokio_runtime, move || stream_provider.is_retriable())
                    .await
            }
            Self::Bytes { .. } => Ok(true),
        }
    }

    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError> {
        match self {
            Self::File { path, .. } => {
                let file = File::open(path).await?;
                let size = SizeInfo::Exact(file.metadata().await?.size() as i64);

                Ok((Box::pin(file.compat()), size))
            }
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                let reader_tokio_runtime = tokio_runtime.clone();

                uploadable_stream_blocking(&tokio_runtime, move || {
                    let size = stream_provider.size()?.into();

                    let stream = stream_provider.stream()?;

                    let reader: BoxAsyncRead =
                        Box::pin(UploadStreamReader::new(stream, reader_tokio_runtime));

                    Ok((reader, size))
                })
                .await
            }
            Self::Bytes { bytes } => Ok((
                Box::pin(Cursor::new(bytes.to_owned())),
                SizeInfo::Exact(bytes.len() as i64),
            )),
        }
    }
}

impl Drop for MobileUploadable {
    fn drop(&mut self) {
        match self {
            Self::File {
                path,
                remove_file_after_upload,
                tokio_runtime,
            } => {
                if *remove_file_after_upload {
                    let path = path.clone();

                    tokio_runtime.spawn(async move {
                        if let Err(err) = tokio::fs::remove_file(path).await {
                            log::warn!("MobileUploadable drop File failed to remove file: {}", err);
                        }
                    });
                }
            }
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                tokio_runtime.spawn_blocking(move || {
                    if let Err(err) = stream_provider.dispose() {
                        log::warn!(
                            "MobileUploadable drop UploadStream failed to dispose: {}",
                            err
                        );
                    }
                });
            }
            Self::Bytes { .. } => {}
        }
    }
}

async fn uploadable_stream_blocking<F, T>(
    tokio_runtime: &tokio::runtime::Runtime,
    f: F,
) -> Result<T, UploadableError>
where
    F: FnOnce() -> Result<T, StreamError> + Send + 'static,
    T: Send + 'static,
{
    match tokio_runtime
        .spawn_blocking(move || f().map_err(|err| Into::<UploadableError>::into(err)))
        .await
    {
        Ok(res) => res,
        Err(err) => Err(UploadableError::LocalFileError(err.to_string())),
    }
}
