use std::pin::Pin;

use futures::{
    stream::{StreamExt, TryStreamExt},
    AsyncRead, AsyncReadExt,
};
use thiserror::Error;
use vault_core::user_error::UserError;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;

use vault_core::cipher::constants::BLOCK_SIZE;

#[wasm_bindgen(module = "/js/helpers.js")]
extern "C" {
    #[wasm_bindgen(js_name = "supportsRequestStreams")]
    pub fn supports_request_streams() -> bool;

    #[wasm_bindgen(js_name = "streamToBlob")]
    pub fn stream_to_blob(stream: JsValue, content_type: Option<&str>) -> js_sys::Promise;

    #[wasm_bindgen(js_name = "supportsReadableByteStream")]
    pub fn supports_readable_byte_stream() -> bool;

    #[wasm_bindgen(js_name = "errorString")]
    pub fn error_string(err: &JsValue) -> String;
}

pub fn bytes_to_array(bytes: &[u8]) -> JsValue {
    let array: js_sys::Uint8Array =
        js_sys::Uint8Array::new_with_length(bytes.len().try_into().unwrap());

    array.copy_from(bytes);

    array.into()
}

pub fn bytes_to_blob(bytes: &[u8], content_type: Option<&str>) -> JsValue {
    let array = bytes_to_array(bytes);

    let blob_parts_array = js_sys::Array::new();

    blob_parts_array.push(&array);

    let mut options = web_sys::BlobPropertyBag::new();

    match content_type {
        Some(content_type) => {
            options.type_(content_type);
        }
        None => {}
    };

    web_sys::Blob::new_with_u8_array_sequence_and_options(&blob_parts_array, &options)
        .unwrap()
        .into()
}

#[derive(Error, Debug, UserError)]
#[error("{0}")]
pub struct ReaderToBlobError(String);

pub async fn reader_to_blob(
    mut reader: Pin<Box<dyn AsyncRead + Send + 'static>>,
    content_type: Option<&str>,
) -> Result<JsValue, ReaderToBlobError> {
    if supports_readable_byte_stream() {
        // it's better to convert readable stream to blob in javascript so that
        // we don't use WASM memory
        let stream = ReadableStream::from_async_read(reader, BLOCK_SIZE).into_raw();
        let stream_value = JsValue::from(stream);

        // TODO handle error
        JsFuture::from(stream_to_blob(stream_value, content_type))
            .await
            .map_err(|err| ReaderToBlobError(error_string(&err)))
    } else {
        let mut buf = Vec::new();

        reader
            .read_to_end(&mut buf)
            .await
            .map_err(|e| ReaderToBlobError(e.to_string()))?;

        Ok(bytes_to_blob(&buf, content_type))
    }
}

pub fn stream_to_reader(
    stream: web_sys::ReadableStream,
) -> Pin<Box<dyn AsyncRead + Send + Sync + 'static>> {
    let stream = ReadableStream::from_raw(stream.unchecked_into()).into_stream();

    let reader = stream
        .map(|chunk| {
            chunk
                .map(|value| value.dyn_into::<js_sys::Uint8Array>().unwrap().to_vec())
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, error_string(&err)))
        })
        .into_async_read();

    Box::into_pin(unsafe {
        Box::from_raw(
            Box::into_raw(Box::new(reader) as Box<dyn AsyncRead + 'static>)
                as *mut (dyn AsyncRead + Send + Sync + 'static),
        )
    })
}

#[derive(Error, Debug, UserError)]
pub enum ReaderToFileStreamError {
    #[error("{0}")]
    ReaderToBlobError(#[from] ReaderToBlobError),
}

pub async fn reader_to_file_stream(
    name: &str,
    reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
    size: Option<i64>,
    content_type: Option<&str>,
    force_blob: bool,
) -> Result<JsValue, ReaderToFileStreamError> {
    let file_stream = js_sys::Object::new();

    js_sys::Reflect::set(&file_stream, &JsValue::from("name"), &JsValue::from(name)).unwrap();

    if let Some(size) = size {
        js_sys::Reflect::set(&file_stream, &JsValue::from("size"), &JsValue::from(size)).unwrap();
    }

    if supports_readable_byte_stream() && !force_blob {
        let stream = ReadableStream::from_async_read(reader, 1024 * 1024);

        js_sys::Reflect::set(
            &file_stream,
            &JsValue::from("stream"),
            &JsValue::from(stream.into_raw()),
        )
        .unwrap();
    } else {
        let blob = reader_to_blob(reader, content_type).await?;

        js_sys::Reflect::set(&file_stream, &JsValue::from("blob"), &JsValue::from(blob)).unwrap();
    }

    Ok(JsValue::from(file_stream))
}
