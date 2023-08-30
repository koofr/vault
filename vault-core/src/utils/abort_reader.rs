use futures::{
    stream::AbortHandle,
    task::{Context, Poll},
    AsyncRead,
};
use pin_project_lite::pin_project;
use std::{io::Result, pin::Pin};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("reader aborted")]
pub struct ReaderAbortedError;

impl Into<std::io::Error> for ReaderAbortedError {
    fn into(self) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Interrupted, self)
    }
}

pin_project! {
    pub struct AbortReader<R> {
        #[pin]
        inner: R,
        abort_handle: AbortHandle
    }
}

impl<R> AbortReader<R> {
    pub fn new(inner: R, abort_handle: AbortHandle) -> Self {
        Self {
            inner,
            abort_handle,
        }
    }
}

impl<R: AsyncRead> AsyncRead for AbortReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        if self.abort_handle.is_aborted() {
            return Poll::Ready(Err(ReaderAbortedError.into()));
        }

        let mut this = self.project();

        this.inner.as_mut().poll_read(cx, buf)
    }
}
