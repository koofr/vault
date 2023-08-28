use async_trait::async_trait;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Blob, File, ReadableStream};

use vault_core::{
    common::state::{BoxAsyncRead, SizeInfo},
    transfers::{errors::UploadableError, uploadable::Uploadable},
};

use crate::helpers;

pub enum BrowserUploadable {
    File(File),
    Blob(Blob),
}

unsafe impl Send for BrowserUploadable {}
unsafe impl Sync for BrowserUploadable {}

impl BrowserUploadable {
    pub fn from_value(value: JsValue) -> Result<Self, String> {
        if value.has_type::<File>() {
            Ok(Self::File(value.unchecked_into()))
        } else if value.has_type::<Blob>() {
            Ok(Self::Blob(value.unchecked_into()))
        } else {
            Err(String::from("expected File or Blob"))
        }
    }

    pub fn size(&self) -> i64 {
        match self {
            Self::File(file) => file.size() as i64,
            Self::Blob(blob) => blob.size() as i64,
        }
    }

    pub fn stream(&self) -> ReadableStream {
        match self {
            Self::File(file) => file.stream(),
            Self::Blob(blob) => blob.stream(),
        }
    }

    pub fn reader(&self) -> BoxAsyncRead {
        helpers::stream_to_reader(self.stream())
    }
}

#[async_trait]
impl Uploadable for BrowserUploadable {
    async fn size(&self) -> Result<SizeInfo, UploadableError> {
        Ok(SizeInfo::Exact(self.size()))
    }

    async fn is_retriable(&self) -> Result<bool, UploadableError> {
        Ok(true)
    }

    async fn reader(&self) -> Result<(BoxAsyncRead, SizeInfo), UploadableError> {
        Ok((self.reader(), SizeInfo::Exact(self.size())))
    }
}
