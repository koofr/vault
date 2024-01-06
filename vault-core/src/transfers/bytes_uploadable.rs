use async_trait::async_trait;
use futures::io::Cursor;

use crate::{
    common::state::{BoxAsyncRead, SizeInfo},
    transfers::{errors::UploadableError, uploadable::Uploadable},
};

pub struct BytesUploadable {
    pub bytes: Vec<u8>,
}

#[async_trait]
impl Uploadable for BytesUploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError> {
        Ok(SizeInfo::Exact(self.bytes.len() as i64))
    }

    async fn is_retriable(&self) -> Result<bool, UploadableError> {
        Ok(true)
    }

    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError> {
        Ok((
            Box::pin(Cursor::new(self.bytes.to_owned())),
            SizeInfo::Exact(self.bytes.len() as i64),
        ))
    }
}
