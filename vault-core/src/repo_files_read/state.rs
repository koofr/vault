use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{
    common::state::{BoxAsyncRead, SizeInfo},
    remote_files::state::RemoteFile,
    repo_files::state::RepoFileType,
    types::{DecryptedName, MountId, RemotePath, RepoId},
};

use super::errors::GetFilesReaderError;

pub struct RepoFileReader {
    pub name: DecryptedName,
    pub size: SizeInfo,
    /// content_type is needed in vault-wasm to build Blobs. without correct
    /// content-type, imgs are not displayed
    pub content_type: Option<String>,
    /// remote_file is needed in repo files details to check if the remote
    /// content has changed by comparing remote (size, modified, hash)
    pub remote_file: Option<RemoteFile>,
    /// unique_name is used for local file caching. it will not be set for
    /// generated files (e.g. ZIP files of a dir)
    pub unique_name: Option<String>,
    pub reader: BoxAsyncRead,
}

impl RepoFileReader {
    pub fn wrap_reader(self, f: impl FnOnce(BoxAsyncRead) -> BoxAsyncRead) -> Self {
        let reader = f(self.reader);

        Self {
            name: self.name,
            size: self.size,
            content_type: self.content_type,
            remote_file: self.remote_file,
            unique_name: self.unique_name,
            reader,
        }
    }
}

/// RepoFileReaderBuilder is Fn() (and not FnOnce()) because download transfers
/// can be retried
pub type RepoFileReaderBuilder = Box<
    dyn Fn() -> BoxFuture<'static, Result<RepoFileReader, GetFilesReaderError>>
        + Send
        + Sync
        + 'static,
>;

pub struct RepoFileReaderProvider {
    pub name: DecryptedName,
    pub size: SizeInfo,
    /// unique_name is used for local file caching. it will not be set for
    /// generated files (e.g. ZIP files of a dir)
    pub unique_name: Option<String>,
    pub reader_builder: RepoFileReaderBuilder,
}

impl RepoFileReaderProvider {
    pub async fn reader(&self) -> Result<RepoFileReader, GetFilesReaderError> {
        (self.reader_builder)().await
    }

    pub fn wrap_reader_builder(
        self,
        f: impl Fn(
                Arc<RepoFileReaderBuilder>,
            ) -> BoxFuture<'static, Result<RepoFileReader, GetFilesReaderError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        let reader_builder = Arc::new(self.reader_builder);
        let f = Arc::new(f);

        Self {
            name: self.name,
            size: self.size,
            unique_name: self.unique_name,
            reader_builder: Box::new(move || f(reader_builder.clone())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteZipEntry {
    pub mount_id: MountId,
    pub remote_path: RemotePath,
    pub repo_id: RepoId,
    /// relative path without leading / (dirs end with /)
    pub filename: String,
    pub modified: async_zip_futures::ZipDateTime,
    pub typ: RepoFileType,
    pub size: i64,
}

pub type RemoteZipEntriesFuture =
    BoxFuture<'static, Result<Vec<RemoteZipEntry>, GetFilesReaderError>>;

pub type GetRemoteZipEntries = Box<dyn Fn() -> RemoteZipEntriesFuture + Send + Sync + 'static>;
