use std::{cmp, pin::Pin, sync::Arc};

use futures::{
    ready,
    task::{Context, Poll},
    AsyncRead, Future,
};
use tokio::{runtime::Runtime, task::JoinHandle};

use crate::{StreamError, UploadStream};

enum UploadStreamReaderState {
    Idle(Option<(Vec<u8>, usize)>),
    Busy(JoinHandle<Result<Vec<u8>, StreamError>>),
    Done,
}

pub struct UploadStreamReader {
    stream: Arc<Box<dyn UploadStream>>,
    runtime: Arc<Runtime>,
    state: UploadStreamReaderState,
}

impl UploadStreamReader {
    pub fn new(stream: Box<dyn UploadStream>, runtime: Arc<Runtime>) -> Self {
        Self {
            stream: Arc::new(stream),
            runtime,
            state: UploadStreamReaderState::Idle(None),
        }
    }
}

impl AsyncRead for UploadStreamReader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        use UploadStreamReaderState::*;

        let this = self.get_mut();

        loop {
            match this.state {
                Idle(ref mut stream_buf_cell) => {
                    let mut written = 0;

                    if let Some((stream_buf, mut pos)) = stream_buf_cell.take() {
                        if pos < stream_buf.len() {
                            let n = cmp::min(buf.len(), stream_buf.len() - pos);

                            buf[..n].copy_from_slice(&stream_buf[pos..pos + n]);

                            pos += n;
                            written += n;

                            if pos < stream_buf.len() {
                                *stream_buf_cell = Some((stream_buf, pos));
                            }
                        }
                    }

                    if stream_buf_cell.is_none() {
                        let stream = this.stream.clone();

                        this.state = Busy(this.runtime.spawn_blocking(move || stream.read()));
                    }

                    if written > 0 {
                        return Poll::Ready(Ok(written));
                    }
                }
                Busy(ref mut rx) => match ready!(Pin::new(rx).poll(cx))? {
                    Ok(buf) => {
                        if buf.is_empty() {
                            this.state = Done;
                            return Poll::Ready(Ok(0));
                        }

                        this.state = Idle(Some((buf, 0)));
                    }
                    Err(e) => {
                        this.state = Done;
                        return Poll::Ready(Err(e.into()));
                    }
                },
                Done => {
                    return Poll::Ready(Ok(0));
                }
            }
        }
    }
}

impl Drop for UploadStreamReader {
    fn drop(&mut self) {
        let _ = self.stream.close();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use futures::AsyncReadExt;
    use tokio::runtime::Runtime;

    use crate::{StreamError, UploadStream};

    use super::UploadStreamReader;

    #[derive(Debug)]
    struct MockUploadStream {
        queue: Arc<Mutex<Vec<Result<Vec<u8>, StreamError>>>>,
    }

    impl UploadStream for MockUploadStream {
        fn read(&self) -> Result<Vec<u8>, StreamError> {
            self.queue.lock().unwrap().pop().unwrap_or(Ok(vec![]))
        }

        fn close(&self) -> Result<(), StreamError> {
            Ok(())
        }
    }

    #[test]
    fn test_read() {
        let runtime = Arc::new(Runtime::new().unwrap());

        let part1 = vec![1; 1024];
        let part2 = vec![2; 1024];
        let part3 = vec![3; 1024];
        let part4 = vec![4; 1024];

        let mut total = vec![];
        total.extend_from_slice(&part1);
        total.extend_from_slice(&part2);
        total.extend_from_slice(&part3);
        total.extend_from_slice(&part4);

        let stream: Box<dyn UploadStream> = Box::new(MockUploadStream {
            queue: Arc::new(Mutex::new(vec![Ok(part4), Ok(part3), Ok(part2), Ok(part1)])),
        });

        runtime.clone().block_on(async move {
            let mut buf = Vec::new();

            let mut reader = UploadStreamReader::new(stream, runtime);

            let n = reader.read_to_end(&mut buf).await.unwrap();

            assert_eq!(n, total.len());
            assert_eq!(buf, total);
        })
    }

    #[test]
    fn test_read_error() {
        let runtime = Arc::new(Runtime::new().unwrap());

        let part1 = vec![1; 1024];
        let part2 = vec![1; 1024];

        let stream: Box<dyn UploadStream> = Box::new(MockUploadStream {
            queue: Arc::new(Mutex::new(vec![
                Ok(part2.clone()),
                Err(StreamError::IoError {
                    reason: "file not found".into(),
                }),
                Ok(part1.clone()),
            ])),
        });

        runtime.clone().block_on(async move {
            let mut buf = Vec::new();

            let mut reader = UploadStreamReader::new(stream, runtime);

            let res = reader.read_to_end(&mut buf).await;

            assert_eq!(res.unwrap_err().to_string(), "file not found");

            assert_eq!(buf, part1);
        })
    }
}
