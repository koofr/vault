use std::{pin::Pin, sync::Arc};

use futures::{
    ready,
    task::{Context, Poll},
    AsyncWrite, Future,
};
use tokio::{runtime::Runtime, task::JoinHandle};

use crate::{DownloadStream, StreamError};

enum DownloadStreamWriterState {
    Idle,
    Busy(JoinHandle<Result<(), StreamError>>),
    Failed(StreamError),
    Closing(JoinHandle<Result<(), StreamError>>, Option<StreamError>),
    Closed,
    CloseFailed(StreamError),
}

pub struct DownloadStreamWriter {
    stream: Arc<Box<dyn DownloadStream>>,
    runtime: Arc<Runtime>,
    state: DownloadStreamWriterState,
}

impl DownloadStreamWriter {
    pub fn new(stream: Box<dyn DownloadStream>, runtime: Arc<Runtime>) -> Self {
        Self {
            stream: Arc::new(stream),
            runtime,
            state: DownloadStreamWriterState::Idle,
        }
    }
}

impl AsyncWrite for DownloadStreamWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        use DownloadStreamWriterState::*;

        let this = self.get_mut();

        loop {
            match &mut this.state {
                Idle => {
                    let stream = this.stream.clone();
                    let buf = buf.to_owned();
                    let written = buf.len();

                    this.state = Busy(this.runtime.spawn_blocking(move || stream.write(buf)));

                    return Poll::Ready(Ok(written));
                }
                Busy(ref mut rx) => match ready!(Pin::new(rx).poll(cx))? {
                    Ok(()) => {
                        this.state = Idle;
                    }
                    Err(err) => {
                        this.state = Failed(err.clone());
                    }
                },
                Failed(err) => {
                    return Poll::Ready(Err(err.clone().into()));
                }
                Closing(_, _) | Closed | CloseFailed(_) => {
                    return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
                }
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        use DownloadStreamWriterState::*;

        let this = self.get_mut();

        loop {
            match &mut this.state {
                Idle => return Poll::Ready(Ok(())),
                Busy(ref mut rx) => match ready!(Pin::new(rx).poll(cx))? {
                    Ok(()) => {
                        this.state = Idle;
                    }
                    Err(err) => {
                        this.state = Failed(err.clone());
                    }
                },
                Failed(err) => {
                    return Poll::Ready(Err(err.clone().into()));
                }
                Closing(_, _) | Closed | CloseFailed(_) => {
                    return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
                }
            }
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        use DownloadStreamWriterState::*;

        let this = self.get_mut();

        loop {
            match &mut this.state {
                Idle => {
                    let stream = this.stream.clone();

                    this.state = Closing(this.runtime.spawn_blocking(move || stream.close()), None);
                }
                Busy(ref mut rx) => match ready!(Pin::new(rx).poll(cx))? {
                    Ok(()) => {
                        this.state = Idle;
                    }
                    Err(err) => {
                        this.state = Failed(err.clone());
                    }
                },
                Failed(err) => {
                    let stream = this.stream.clone();

                    this.state = Closing(
                        this.runtime.spawn_blocking(move || stream.close()),
                        Some(err.clone()),
                    );
                }
                Closing(ref mut rx, write_err) => {
                    match (ready!(Pin::new(rx).poll(cx))?, write_err.clone()) {
                        (Ok(()), None) => {
                            this.state = Closed;
                        }
                        (Ok(()), Some(write_err)) => {
                            this.state = CloseFailed(write_err.clone());

                            return Poll::Ready(Err(write_err.into()));
                        }
                        (Err(err), _) => {
                            this.state = CloseFailed(err.clone());

                            return Poll::Ready(Err(err.into()));
                        }
                    }
                }
                Closed => {
                    return Poll::Ready(Ok(()));
                }
                CloseFailed(err) => {
                    return Poll::Ready(Err(err.clone().into()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use futures::AsyncWriteExt;
    use tokio::runtime::Runtime;

    use crate::{DownloadStream, StreamError};

    use super::DownloadStreamWriter;

    #[derive(Debug)]
    struct MockDownloadStream {
        write_buffers: Arc<Mutex<Vec<Vec<u8>>>>,
        write_res_queue: Arc<Mutex<Vec<Result<(), StreamError>>>>,
        close_res_queue: Arc<Mutex<Vec<Result<(), StreamError>>>>,
    }

    impl DownloadStream for MockDownloadStream {
        fn write(&self, buf: Vec<u8>) -> Result<(), StreamError> {
            self.write_buffers.lock().unwrap().push(buf);

            self.write_res_queue.lock().unwrap().pop().unwrap_or(Ok(()))
        }

        fn close(&self) -> Result<(), StreamError> {
            self.close_res_queue.lock().unwrap().pop().unwrap_or(Ok(()))
        }
    }

    fn fixture() -> (
        Box<dyn DownloadStream>,
        Arc<Mutex<Vec<Vec<u8>>>>,
        Arc<Mutex<Vec<Result<(), StreamError>>>>,
        Arc<Mutex<Vec<Result<(), StreamError>>>>,
    ) {
        let write_buffers = Arc::new(Mutex::new(vec![]));
        let write_res_queue = Arc::new(Mutex::new(vec![]));
        let close_res_queue = Arc::new(Mutex::new(vec![]));

        let stream: Box<dyn DownloadStream> = Box::new(MockDownloadStream {
            write_buffers: write_buffers.clone(),
            write_res_queue: write_res_queue.clone(),
            close_res_queue: close_res_queue.clone(),
        });

        (stream, write_buffers, write_res_queue, close_res_queue)
    }

    #[test]
    fn test_write_close() {
        let runtime = Arc::new(Runtime::new().unwrap());

        runtime.clone().block_on(async move {
            let (stream, write_buffers, _, _) = fixture();

            let mut writer = DownloadStreamWriter::new(stream, runtime);

            let part1 = vec![1; 1024];
            let part2 = vec![2; 1024];
            let part3 = vec![3; 1024];
            let part4 = vec![4; 1024];

            let n = writer.write(&part1).await.unwrap();
            assert_eq!(n, part1.len());
            let n = writer.write(&part2).await.unwrap();
            assert_eq!(n, part2.len());
            let n = writer.write(&part3).await.unwrap();
            assert_eq!(n, part3.len());
            let n = writer.write(&part4).await.unwrap();
            assert_eq!(n, part4.len());

            writer.close().await.unwrap();

            assert_eq!(
                write_buffers.lock().unwrap().to_owned(),
                vec![part1, part2, part3, part4]
            );
        })
    }

    #[test]
    fn test_write_error_close() {
        let runtime = Arc::new(Runtime::new().unwrap());

        runtime.clone().block_on(async move {
            let (stream, write_buffers, write_res_queue, _) = fixture();

            write_res_queue
                .lock()
                .unwrap()
                .push(Err(StreamError::IoError {
                    reason: "write failed".into(),
                }));
            write_res_queue.lock().unwrap().push(Ok(()));

            let mut writer = DownloadStreamWriter::new(stream, runtime);

            let part1 = vec![1; 1024];
            let part2 = vec![2; 1024];

            let n = writer.write(&part1).await.unwrap();
            assert_eq!(n, part1.len());
            let res = writer.write(&part2).await;
            assert!(res.is_ok());

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            assert_eq!(write_buffers.lock().unwrap().to_owned(), vec![part1, part2]);
        })
    }

    #[test]
    fn test_write_error_write_close() {
        let runtime = Arc::new(Runtime::new().unwrap());

        runtime.clone().block_on(async move {
            let (stream, write_buffers, write_res_queue, _) = fixture();

            write_res_queue
                .lock()
                .unwrap()
                .push(Err(StreamError::IoError {
                    reason: "write failed".into(),
                }));
            write_res_queue.lock().unwrap().push(Ok(()));

            let mut writer = DownloadStreamWriter::new(stream, runtime);

            let part1 = vec![1; 1024];
            let part2 = vec![2; 1024];
            let part3 = vec![3; 1024];

            let n = writer.write(&part1).await.unwrap();
            assert_eq!(n, part1.len());
            let n = writer.write(&part2).await.unwrap();
            assert_eq!(n, part2.len());
            let res = writer.write(&part3).await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            assert_eq!(write_buffers.lock().unwrap().to_owned(), vec![part1, part2]);
        })
    }

    #[test]
    fn test_write_error_flush_close() {
        let runtime = Arc::new(Runtime::new().unwrap());

        runtime.clone().block_on(async move {
            let (stream, write_buffers, write_res_queue, _) = fixture();

            write_res_queue
                .lock()
                .unwrap()
                .push(Err(StreamError::IoError {
                    reason: "write failed".into(),
                }));
            write_res_queue.lock().unwrap().push(Ok(()));

            let mut writer = DownloadStreamWriter::new(stream, runtime);

            let part1 = vec![1; 1024];
            let part2 = vec![2; 1024];

            let n = writer.write(&part1).await.unwrap();
            assert_eq!(n, part1.len());
            let n = writer.write(&part2).await.unwrap();
            assert_eq!(n, part2.len());
            let res = writer.flush().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "write failed");

            assert_eq!(write_buffers.lock().unwrap().to_owned(), vec![part1, part2]);
        })
    }

    #[test]
    fn test_write_error_close_error() {
        let runtime = Arc::new(Runtime::new().unwrap());

        runtime.clone().block_on(async move {
            let (stream, write_buffers, write_res_queue, close_res_queue) = fixture();

            write_res_queue
                .lock()
                .unwrap()
                .push(Err(StreamError::IoError {
                    reason: "write failed".into(),
                }));
            write_res_queue.lock().unwrap().push(Ok(()));

            close_res_queue
                .lock()
                .unwrap()
                .push(Err(StreamError::IoError {
                    reason: "close failed".into(),
                }));

            let mut writer = DownloadStreamWriter::new(stream, runtime);

            let part1 = vec![1; 1024];
            let part2 = vec![2; 1024];

            let n = writer.write(&part1).await.unwrap();
            assert_eq!(n, part1.len());
            let res = writer.write(&part2).await;
            assert!(res.is_ok());

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "close failed");

            let res = writer.close().await;
            assert!(res.is_err());
            assert_eq!(res.unwrap_err().to_string(), "close failed");

            assert_eq!(write_buffers.lock().unwrap().to_owned(), vec![part1, part2]);
        })
    }
}
