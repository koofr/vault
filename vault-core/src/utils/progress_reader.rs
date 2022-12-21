use futures::task::{Context, Poll};
use futures::{ready, AsyncRead};
use pin_project_lite::pin_project;
use std::io::Result;
use std::pin::Pin;

pin_project! {
    pub struct ProgressReader<R> {
        #[pin]
        inner: R,
        on_progress: Box<dyn Fn(usize) + Send + Sync>
    }
}

impl<R> ProgressReader<R> {
    pub fn new(inner: R, on_progress: Box<dyn Fn(usize) + Send + Sync>) -> Self {
        Self { inner, on_progress }
    }
}

impl<R: AsyncRead> AsyncRead for ProgressReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let mut this = self.project();

        let n = ready!(this.inner.as_mut().poll_read(cx, buf))?;

        if n > 0 {
            (this.on_progress)(n);
        }

        Poll::Ready(Ok(n))
    }
}
