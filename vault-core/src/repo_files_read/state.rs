use std::pin::Pin;

use futures::AsyncRead;

use crate::repo_files::state::RepoFileType;

pub struct RepoFileReader {
    pub name: String,
    pub size: Option<i64>,
    pub reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteZipEntry {
    pub mount_id: String,
    pub remote_path: String,
    pub repo_id: String,
    // relative path without leading / (dirs end with /)
    pub filename: String,
    pub modified: async_zip_futures::ZipDateTime,
    pub typ: RepoFileType,
}
