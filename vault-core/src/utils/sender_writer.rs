use std::{
    io::{Error, ErrorKind},
    pin::Pin,
    task::{Context, Poll},
};

use futures::{channel::mpsc::Sender, ready, AsyncWrite};
use pin_project_lite::pin_project;

pin_project! {
    #[derive(Debug)]
    pub struct SenderWriter {
        #[pin]
        sender: Sender<std::io::Result<Vec<u8>>>,
    }
}

impl SenderWriter {
    pub fn new(sender: Sender<std::io::Result<Vec<u8>>>) -> Self {
        Self { sender }
    }
}

impl AsyncWrite for SenderWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let mut this = self.project();

        let len = buf.len();

        ready!(this.sender.poll_ready(cx))
            .and_then(|_| this.sender.start_send(Ok(buf.to_vec())))
            .map_err(|err| Error::new(ErrorKind::BrokenPipe, err))?;

        Poll::Ready(Ok(len))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let mut this = self.project();

        this.sender.disconnect();

        Poll::Ready(Ok(()))
    }
}
