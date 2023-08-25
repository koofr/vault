use std::{
    io::{Cursor, ErrorKind, Result, Write},
    pin::Pin,
    task::{Context, Poll},
};

use futures::AsyncWrite;

pub struct MemoryWriter {
    cursor: Option<Cursor<Vec<u8>>>,
    on_close: Option<Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>>,
}

impl MemoryWriter {
    pub fn new(on_close: Box<dyn Fn(Vec<u8>) + Send + Sync + 'static>) -> Self {
        let cursor = Cursor::new(Vec::new());

        Self {
            cursor: Some(cursor),
            on_close: Some(on_close),
        }
    }
}

impl AsyncWrite for MemoryWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        match &mut self.cursor {
            Some(cursor) => Poll::Ready(cursor.write(buf)),
            None => Poll::Ready(Err(ErrorKind::BrokenPipe.into())),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match &mut self.cursor {
            Some(cursor) => Poll::Ready(cursor.flush()),
            None => Poll::Ready(Err(ErrorKind::BrokenPipe.into())),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.cursor.take() {
            Some(mut cursor) => match cursor.flush() {
                Ok(()) => match self.on_close.take() {
                    Some(on_close) => {
                        on_close(cursor.into_inner());

                        Poll::Ready(Ok(()))
                    }
                    None => Poll::Ready(Err(ErrorKind::BrokenPipe.into())),
                },
                Err(err) => Poll::Ready(Err(err)),
            },
            None => Poll::Ready(Err(ErrorKind::BrokenPipe.into())),
        }
    }
}
