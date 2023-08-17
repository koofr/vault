use futures::{
    ready,
    task::{Context, Poll},
    AsyncRead,
};
use pin_project_lite::pin_project;
use std::{io::Result, pin::Pin};

pin_project! {
    pub struct MD5Reader<R> {
        #[pin]
        inner: R,
        md5_context: md5::Context,
    }
}

impl<R> MD5Reader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            md5_context: md5::Context::new(),
        }
    }

    pub fn digest(self) -> md5::Digest {
        self.md5_context.compute()
    }

    pub fn hex_digest(self) -> String {
        format!("{:x}", self.digest())
    }
}

impl<R: AsyncRead> AsyncRead for MD5Reader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let mut this = self.project();

        let n = ready!(this.inner.as_mut().poll_read(cx, buf))?;

        if n > 0 {
            this.md5_context.consume(&buf[..n]);
        }

        Poll::Ready(Ok(n))
    }
}
