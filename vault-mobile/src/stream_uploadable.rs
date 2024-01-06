use std::sync::Arc;

use async_trait::async_trait;

use vault_core::{
    common::state::{BoxAsyncRead, SizeInfo},
    transfers::{errors::UploadableError, uploadable::Uploadable},
};

use crate::{upload_stream_reader::UploadStreamReader, StreamError, UploadStreamProvider};

pub struct StreamUploadable {
    pub stream_provider: Arc<Box<dyn UploadStreamProvider>>,
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
}

impl StreamUploadable {
    async fn blocking<F, T>(&self, f: F) -> Result<T, UploadableError>
    where
        F: FnOnce(Arc<Box<dyn UploadStreamProvider>>) -> Result<T, StreamError> + Send + 'static,
        T: Send + 'static,
    {
        let stream_provider = self.stream_provider.clone();

        match self
            .tokio_runtime
            .spawn_blocking(move || {
                f(stream_provider).map_err(|err| Into::<UploadableError>::into(err))
            })
            .await
        {
            Ok(res) => res,
            Err(err) => Err(UploadableError::LocalFileError(err.to_string())),
        }
    }
}

#[async_trait]
impl Uploadable for StreamUploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError> {
        self.blocking(move |stream_provider| Ok(stream_provider.size()?.into()))
            .await
    }

    async fn is_retriable(&self) -> Result<bool, UploadableError> {
        self.blocking(move |stream_provider| stream_provider.is_retriable())
            .await
    }

    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError> {
        let reader_tokio_runtime = self.tokio_runtime.clone();

        self.blocking(move |stream_provider| {
            let size = stream_provider.size()?.into();

            let stream = stream_provider.stream()?;

            let reader: BoxAsyncRead =
                Box::pin(UploadStreamReader::new(stream, reader_tokio_runtime));

            Ok((reader, size))
        })
        .await
    }
}

impl Drop for StreamUploadable {
    fn drop(&mut self) {
        let stream_provider = self.stream_provider.clone();

        self.tokio_runtime.spawn_blocking(move || {
            if let Err(err) = stream_provider.dispose() {
                log::warn!(
                    "MobileUploadable drop UploadStream failed to dispose: {}",
                    err
                );
            }
        });
    }
}
