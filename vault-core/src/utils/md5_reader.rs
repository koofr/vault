use futures::{
    channel::oneshot,
    ready,
    task::{Context, Poll},
    AsyncRead,
};
use pin_project_lite::pin_project;
use std::{io::Result, pin::Pin};

struct Container {
    md5_context: md5::Context,
    sender: Option<oneshot::Sender<md5::Digest>>,
}

impl Drop for Container {
    fn drop(&mut self) {
        let digest = self.md5_context.clone().compute();

        if let Some(sender) = self.sender.take() {
            let _ = sender.send(digest);
        }
    }
}

pin_project! {
    pub struct MD5Reader<R> {
        #[pin]
        inner: R,
        container: Container,
    }
}

impl<R> MD5Reader<R> {
    pub fn new(inner: R) -> (Self, oneshot::Receiver<md5::Digest>) {
        let (sender, receiver) = oneshot::channel();

        (
            Self {
                inner,
                container: Container {
                    md5_context: md5::Context::new(),
                    sender: Some(sender),
                },
            },
            receiver,
        )
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
            this.container.md5_context.consume(&buf[..n]);
        }

        Poll::Ready(Ok(n))
    }
}
