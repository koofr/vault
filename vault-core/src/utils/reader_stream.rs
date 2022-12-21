use bytes::Bytes;
use futures::stream::Stream;
use futures::AsyncRead;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    #[derive(Debug)]
    pub struct ReaderStream<R> {
        #[pin]
        reader: Option<R>,
        buffer_len: usize,
    }
}

impl<R: AsyncRead> ReaderStream<R> {
    pub fn new(reader: R, buffer_len: usize) -> Self {
        ReaderStream {
            reader: Some(reader),
            buffer_len,
        }
    }
}

impl<R: AsyncRead> Stream for ReaderStream<R> {
    type Item = std::io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        let reader = match this.reader.as_pin_mut() {
            Some(r) => r,
            None => return Poll::Ready(None),
        };

        let mut buf = vec![0; *this.buffer_len];

        match reader.poll_read(cx, &mut buf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(err)) => {
                self.project().reader.set(None);
                Poll::Ready(Some(Err(err)))
            }
            Poll::Ready(Ok(0)) => {
                self.project().reader.set(None);
                Poll::Ready(None)
            }
            Poll::Ready(Ok(n)) => Poll::Ready(Some(Ok(buf[..n].to_vec().into()))),
        }
    }
}
