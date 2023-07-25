use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

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
    utils::name_utils,
};

use crate::{
    download_stream_writer::DownloadStreamWriter, DownloadStreamProvider, StreamError,
    TransfersDownloadDone, TransfersDownloadOpen,
};

pub enum MobileDownloadable {
    File {
        original_path: PathBuf,
        append_name: bool,
        autorename: bool,
        on_open: Option<Box<dyn TransfersDownloadOpen>>,
        on_done: Box<dyn TransfersDownloadDone>,
        path: Option<PathBuf>,
        content_type: Option<String>,
    },
    TempFile {
        base_path: PathBuf,
        on_open: Option<Box<dyn TransfersDownloadOpen>>,
        on_done: Box<dyn TransfersDownloadDone>,
        parent_path: Option<PathBuf>,
        /// file is first written to temp_path and then renamed to path
        temp_path: Option<PathBuf>,
        path: Option<PathBuf>,
        content_type: Option<String>,
    },
    Stream {
        stream_provider: Arc<Box<dyn DownloadStreamProvider>>,
        tokio_runtime: Arc<tokio::runtime::Runtime>,
    },
}

#[async_trait]
impl Downloadable for MobileDownloadable {
    async fn is_retriable(&self) -> Result<bool, DownloadableError> {
        match self {
            Self::File { .. } => Ok(true),
            Self::TempFile { .. } => Ok(true),
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                downloadable_stream_blocking(&tokio_runtime, move || stream_provider.is_retriable())
                    .await
            }
        }
    }

    async fn is_openable(&self) -> Result<bool, DownloadableError> {
        match self {
            Self::File { on_open, .. } => Ok(on_open.is_some()),
            Self::TempFile { on_open, .. } => Ok(on_open.is_some()),
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                downloadable_stream_blocking(&tokio_runtime, move || stream_provider.is_openable())
                    .await
            }
        }
    }

    async fn exists(
        &mut self,
        name: String,
        unique_name: String,
    ) -> Result<bool, DownloadableError> {
        match self {
            Self::File { .. } => Ok(false),
            Self::TempFile {
                base_path,
                ref mut parent_path,
                ref mut path,
                ..
            } => {
                let local_parent_path = base_path.join(unique_name);
                let local_path = local_parent_path.join(&name);

                let exists = fs::try_exists(&local_path).await?;

                if exists {
                    *parent_path = Some(local_parent_path);
                    *path = Some(local_path);
                }

                Ok(exists)
            }
            Self::Stream { .. } => Ok(false),
        }
    }

    async fn writer(
        &mut self,
        name: String,
        size: SizeInfo,
        content_type: Option<String>,
        unique_name: Option<String>,
    ) -> Result<(BoxAsyncWrite, String), DownloadableError> {
        match self {
            Self::File {
                original_path,
                append_name,
                autorename,
                path,
                content_type: file_content_type,
                ..
            } => {
                let name = cleanup_name(&name);

                let path_buf = if *append_name {
                    original_path.join(name)
                } else {
                    original_path.clone()
                };

                let (file, path_buf, name) = if *autorename {
                    create_unused_file(path_buf).await?
                } else {
                    let name = path_buf
                        .file_name()
                        .and_then(|name| name.to_str())
                        .map(str::to_string)
                        .ok_or_else(|| {
                            DownloadableError::from(std::io::Error::from(
                                std::io::ErrorKind::InvalidInput,
                            ))
                        })?;

                    (fs::File::create(&path_buf).await?, path_buf, name)
                };

                *path = Some(path_buf);

                *file_content_type = content_type;

                Ok((Box::pin(file.compat()), name))
            }
            Self::TempFile {
                base_path,
                parent_path,
                temp_path,
                path,
                content_type: file_content_type,
                ..
            } => {
                let name = cleanup_name(&name);

                let parent_name = unique_name.clone().unwrap_or(Uuid::new_v4().to_string());

                let local_parent_path = Path::new(base_path).join(parent_name);
                fs::create_dir_all(&local_parent_path).await?;
                *parent_path = Some(local_parent_path.clone());

                let local_temp_path = local_parent_path.join(&Uuid::new_v4().to_string());
                let file = fs::File::create(&local_temp_path).await?;
                *temp_path = Some(local_temp_path);

                let local_path = local_parent_path.join(&name);
                *path = Some(local_path);

                *file_content_type = content_type;

                Ok((Box::pin(file.compat()), name))
            }
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let name = cleanup_name(&name);

                let stream_provider = stream_provider.clone();

                let writer_tokio_runtime = tokio_runtime.clone();

                downloadable_stream_blocking(&tokio_runtime, move || {
                    let stream = stream_provider.stream(
                        name.clone(),
                        size.into(),
                        content_type,
                        unique_name,
                    )?;

                    let writer: BoxAsyncWrite =
                        Box::pin(DownloadStreamWriter::new(stream, writer_tokio_runtime));

                    Ok((writer, name))
                })
                .await
            }
        }
    }

    async fn done(
        &self,
        res: Result<DownloadableStatus, DownloadableError>,
    ) -> Result<(), DownloadableError> {
        match self {
            Self::File {
                on_done,
                path,
                content_type,
                ..
            } => {
                if res.is_ok() {
                    if let Some(path) = path {
                        on_done.on_done(
                            path.to_str().map(str::to_string).ok_or_else(|| {
                                std::io::Error::from(std::io::ErrorKind::InvalidInput)
                            })?,
                            content_type.clone(),
                        );
                    }
                }

                Ok(())
            }
            Self::TempFile {
                on_done,
                parent_path,
                temp_path,
                path,
                content_type,
                ..
            } => {
                if let Some(parent_path) = parent_path {
                    if res.is_ok() {
                        if let Some(path) = path {
                            if let Some(temp_path) = temp_path {
                                fs::rename(temp_path, path).await?;
                            }

                            on_done.on_done(
                                path.to_str().map(str::to_string).ok_or_else(|| {
                                    std::io::Error::from(std::io::ErrorKind::InvalidInput)
                                })?,
                                content_type.clone(),
                            );
                        }
                    } else {
                        let _ = fs::remove_dir_all(parent_path);
                    }
                }

                Ok(())
            }
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();
                let err = res.err();

                downloadable_stream_blocking(&tokio_runtime, move || {
                    stream_provider.done(err.map(|err| err.to_string()))
                })
                .await
            }
        }
    }

    async fn open(&self) -> Result<(), DownloadableError> {
        match self {
            Self::File {
                on_open,
                path,
                content_type,
                ..
            } => match (on_open, path) {
                (Some(on_open), Some(path)) => Ok(on_open.on_open(
                    path.to_str()
                        .map(str::to_string)
                        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?,
                    content_type.clone(),
                )),
                _ => Err(DownloadableError::NotOpenable),
            },
            Self::TempFile {
                on_open,
                path,
                content_type,
                ..
            } => match (on_open, path) {
                (Some(on_open), Some(path)) => Ok(on_open.on_open(
                    path.to_str()
                        .map(str::to_string)
                        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?,
                    content_type.clone(),
                )),
                _ => Err(DownloadableError::NotOpenable),
            },
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                downloadable_stream_blocking(&tokio_runtime, move || stream_provider.open()).await
            }
        }
    }
}

impl Drop for MobileDownloadable {
    fn drop(&mut self) {
        match self {
            Self::File { .. } => {}
            Self::TempFile { .. } => {}
            Self::Stream {
                stream_provider,
                tokio_runtime,
            } => {
                let stream_provider = stream_provider.clone();

                tokio_runtime.spawn_blocking(move || {
                    if let Err(err) = stream_provider.dispose() {
                        log::warn!(
                            "MobileDownloadable drop DownloadStream failed to dispose: {}",
                            err
                        );
                    }
                });
            }
        }
    }
}

async fn downloadable_stream_blocking<F, T>(
    tokio_runtime: &tokio::runtime::Runtime,
    f: F,
) -> Result<T, DownloadableError>
where
    F: FnOnce() -> Result<T, StreamError> + Send + 'static,
    T: Send + 'static,
{
    match tokio_runtime
        .spawn_blocking(move || f().map_err(|err| Into::<DownloadableError>::into(err)))
        .await
    {
        Ok(res) => res,
        Err(err) => Err(DownloadableError::LocalFileError(err.to_string())),
    }
}

async fn create_unused_file(path: PathBuf) -> std::io::Result<(fs::File, PathBuf, String)> {
    let parent_path = path
        .parent()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;

    let (base_name, ext) = name_utils::split_name_ext(&name);

    let mut i = 0;

    loop {
        let new_name = if i == 0 {
            name.clone()
        } else {
            name_utils::join_name_ext(&format!("{} ({})", base_name, i), ext)
        };

        i += 1;

        let path = parent_path.join(&new_name);

        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(file) => return Ok((file, path, new_name)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }
}

pub fn cleanup_name(name: &str) -> String {
    let name = name.replace(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'], "");

    match name.as_str() {
        "" => "invalid name".into(),
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use super::cleanup_name;

    #[test]
    pub fn test_cleanup_name() {
        assert_eq!(cleanup_name("file.txt"), "file.txt");
        assert_eq!(cleanup_name("file <1>.txt"), "file 1.txt");
        assert_eq!(cleanup_name("foo:bar.txt"), "foobar.txt");
        assert_eq!(cleanup_name("\"/\\|?*"), "invalid name");
    }
}
