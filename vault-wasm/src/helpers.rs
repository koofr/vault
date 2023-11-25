use std::sync::Arc;

use futures::{
    stream::{StreamExt, TryStreamExt},
    AsyncRead, AsyncReadExt,
};
use thiserror::Error;
use vault_crypto::constants::BLOCK_SIZE;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_streams::ReadableStream;
use web_sys::AbortSignal;

use vault_core::{
    common::state::{BoxAsyncRead, SizeInfo},
    user_error::UserError,
    utils::on_end_reader::OnEndReader,
    Vault,
};

use crate::dto;

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

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0}")]
pub struct ReaderToBlobError(String);

impl UserError for ReaderToBlobError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

pub async fn reader_to_blob(
    mut reader: BoxAsyncRead,
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

pub fn stream_to_reader(stream: web_sys::ReadableStream) -> BoxAsyncRead {
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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ReaderToFileStreamError {
    #[error("{0}")]
    ReaderToBlobError(#[from] ReaderToBlobError),
}

impl UserError for ReaderToFileStreamError {
    fn user_error(&self) -> String {
        self.to_string()
    }
}

pub async fn reader_to_file_stream(
    name: &str,
    reader: BoxAsyncRead,
    size: SizeInfo,
    content_type: Option<&str>,
    force_blob: bool,
) -> Result<JsValue, ReaderToFileStreamError> {
    let file_stream = js_sys::Object::new();

    js_sys::Reflect::set(&file_stream, &JsValue::from("name"), &JsValue::from(name)).unwrap();

    js_sys::Reflect::set(
        &file_stream,
        &JsValue::from("size"),
        &JsValue::from(serde_wasm_bindgen::to_value(&Into::<dto::SizeInfo>::into(&size)).unwrap()),
    )
    .unwrap();

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

pub fn transfers_download_reader_abort_signal(
    vault: Arc<Vault>,
    reader: BoxAsyncRead,
    transfer_id: u32,
    abort_signal: AbortSignal,
) -> BoxAsyncRead {
    let on_abort_closure = Closure::<dyn Fn() + 'static>::new(move || {
        vault.transfers_abort(transfer_id);
    });

    abort_signal.set_onabort(Some(on_abort_closure.as_ref().unchecked_ref()));

    let cleanup: Box<dyn FnOnce() + 'static> = Box::new(move || {
        abort_signal.set_onabort(None);
        // drop the close on reader end so that we don't leak the memory
        drop(on_abort_closure);
    });
    let cleanup = unsafe {
        Box::from_raw(
            Box::into_raw(Box::new(cleanup) as Box<dyn FnOnce() + 'static>)
                as *mut (dyn FnOnce() + Send + Sync + 'static),
        )
    };

    Box::pin(OnEndReader::new(
        reader,
        Box::new(move |_| {
            cleanup();
        }),
    ))
}
