use std::{fmt::Debug, ops::RangeInclusive, pin::Pin, sync::Arc};

use async_trait::async_trait;
use futures::AsyncRead;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ObjectProviderError {
    #[error("{0}")]
    IOError(Arc<std::io::Error>),
}

impl From<std::io::Error> for ObjectProviderError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(Arc::new(err))
    }
}

#[async_trait]
pub trait ObjectProvider: Debug {
    async fn get(
        &self,
        object_id: String,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<Pin<Box<dyn AsyncRead + Send + Sync + 'static>>, ObjectProviderError>;
    async fn put(
        &self,
        object_id: String,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
    ) -> Result<u64, ObjectProviderError>;
    async fn delete(&self, object_id: String) -> Result<(), ObjectProviderError>;
}

pub type BoxObjectProvider = Box<dyn ObjectProvider + Send + Sync>;
