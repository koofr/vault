use std::{path::PathBuf, sync::Arc};

use futures::future::BoxFuture;

#[derive(Clone)]
pub struct FileHandlers {
    pub pick_files: Option<
        Arc<Box<dyn Fn() -> BoxFuture<'static, Option<Vec<PathBuf>>> + Send + Sync + 'static>>,
    >,
    pub pick_dirs: Option<
        Arc<Box<dyn Fn() -> BoxFuture<'static, Option<Vec<PathBuf>>> + Send + Sync + 'static>>,
    >,
    pub save_file: Option<
        Arc<Box<dyn Fn(String) -> BoxFuture<'static, Option<PathBuf>> + Send + Sync + 'static>>,
    >,
}
