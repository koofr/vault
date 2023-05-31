use std::pin::Pin;

use futures::AsyncRead;

use crate::{common::state::SizeInfo, remote::models, repo_files::state::RepoFileType};

pub struct RepoFileReader {
    pub name: String,
    pub size: SizeInfo,
    pub content_type: Option<String>,
    pub remote_file: Option<models::FilesFile>,
    pub reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
}

impl RepoFileReader {
    pub fn wrap_reader(
        self,
        f: impl FnOnce(
            Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
        ) -> Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
    ) -> Self {
        let Self {
            name,
            size,
            content_type,
            remote_file,
            reader,
        } = self;

        let reader = f(reader);

        Self {
            name,
            size,
            content_type,
            remote_file,
            reader,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteZipEntry {
    pub mount_id: String,
    pub remote_path: String,
    pub repo_id: String,
    /// relative path without leading / (dirs end with /)
    pub filename: String,
    pub modified: async_zip_futures::ZipDateTime,
    pub typ: RepoFileType,
    pub size: i64,
}
