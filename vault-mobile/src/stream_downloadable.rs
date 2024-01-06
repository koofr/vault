use std::sync::Arc;

use async_trait::async_trait;

use vault_core::{
    common::state::{BoxAsyncWrite, SizeInfo},
    transfers::{
        downloadable::{Downloadable, DownloadableStatus},
        errors::DownloadableError,
    },
};
use vault_native::file_utils;

use crate::{download_stream_writer::DownloadStreamWriter, DownloadStreamProvider, StreamError};

pub struct StreamDownloadable {
    pub stream_provider: Arc<Box<dyn DownloadStreamProvider>>,
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,
}

impl StreamDownloadable {
    async fn blocking<F, T>(&self, f: F) -> Result<T, DownloadableError>
    where
        F: FnOnce(Arc<Box<dyn DownloadStreamProvider>>) -> Result<T, StreamError> + Send + 'static,
        T: Send + 'static,
    {
        let stream_provider = self.stream_provider.clone();

        match self
            .tokio_runtime
            .spawn_blocking(move || {
                f(stream_provider).map_err(|err| Into::<DownloadableError>::into(err))
            })
            .await
        {
            Ok(res) => res,
            Err(err) => Err(DownloadableError::LocalFileError(err.to_string())),
        }
    }
}

#[async_trait]
impl Downloadable for StreamDownloadable {
    async fn is_retriable(&self) -> Result<bool, DownloadableError> {
        self.blocking(move |stream_provider| stream_provider.is_retriable())
            .await
    }

    async fn is_openable(&self) -> Result<bool, DownloadableError> {
        self.blocking(move |stream_provider| stream_provider.is_openable())
            .await
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
        size: SizeInfo,
        content_type: Option<String>,
        unique_name: Option<String>,
    ) -> Result<(BoxAsyncWrite, String), DownloadableError> {
        let name = file_utils::cleanup_name(&name);

        let writer_tokio_runtime = self.tokio_runtime.clone();

        self.blocking(move |stream_provider| {
            let stream =
                stream_provider.stream(name.clone(), size.into(), content_type, unique_name)?;

            let writer: BoxAsyncWrite =
                Box::pin(DownloadStreamWriter::new(stream, writer_tokio_runtime));

            Ok((writer, name))
        })
        .await
    }

    async fn done(
        &self,
        res: Result<DownloadableStatus, DownloadableError>,
    ) -> Result<(), DownloadableError> {
        let err = res.err();

        self.blocking(move |stream_provider| stream_provider.done(err.map(|err| err.to_string())))
            .await
    }

    async fn open(&self) -> Result<(), DownloadableError> {
        self.blocking(move |stream_provider| stream_provider.open())
            .await
    }
}

impl Drop for StreamDownloadable {
    fn drop(&mut self) {
        let stream_provider = self.stream_provider.clone();

        self.tokio_runtime.spawn_blocking(move || {
            if let Err(err) = stream_provider.dispose() {
                log::warn!(
                    "MobileDownloadable drop DownloadStream failed to dispose: {}",
                    err
                );
            }
        });
    }
}
