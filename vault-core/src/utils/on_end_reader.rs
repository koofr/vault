use futures::{
    ready,
    task::{Context, Poll},
    AsyncRead,
};
use pin_project_lite::pin_project;
use std::pin::Pin;

pin_project! {
    pub struct OnEndReader<R> {
        #[pin]
        inner: R,
        on_end: Option<Box<dyn FnOnce(Result<(), &std::io::Error>) + Send + Sync>>
    }
}

impl<R> OnEndReader<R> {
    pub fn new(
        inner: R,
        on_end: Box<dyn FnOnce(Result<(), &std::io::Error>) + Send + Sync>,
    ) -> Self {
        Self {
            inner,
            on_end: Some(on_end),
        }
    }
}

impl<R: AsyncRead> AsyncRead for OnEndReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let mut this = self.project();

        let res = ready!(this.inner.as_mut().poll_read(cx, buf));

        match &res {
            Ok(n) if *n == 0 => {
                if let Some(on_end) = this.on_end.take() {
                    on_end(Ok(()))
                }
            }
            Err(err) => {
                if let Some(on_end) = this.on_end.take() {
                    on_end(Err(err))
                }
            }
            _ => {}
        }

        Poll::Ready(res)
    }
}
