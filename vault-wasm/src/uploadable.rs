use std::pin::Pin;

use futures::AsyncRead;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Blob, File, ReadableStream};

use crate::helpers;

pub enum Uploadable {
    File(File),
    Blob(Blob),
}

unsafe impl Send for Uploadable {}
unsafe impl Sync for Uploadable {}

impl Uploadable {
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

    pub fn reader(&self) -> Pin<Box<dyn AsyncRead + Send + Sync + 'static>> {
        helpers::stream_to_reader(self.stream())
    }
}

impl vault_core::repo_files::state::RepoFileUploadable for Uploadable {
    fn size(&self) -> Option<i64> {
        Some(self.size())
    }

    fn reader(&self) -> Pin<Box<dyn AsyncRead + Send + Sync + 'static>> {
        self.reader()
    }
}
