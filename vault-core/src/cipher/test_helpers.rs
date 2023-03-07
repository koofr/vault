use super::Cipher;

#[macro_export]
macro_rules! assert_reader_pending {
    ($reader:expr) => {{
        assert_reader_pending!($reader, 100000)
    }};
    ($reader:expr, $buf_size:expr) => {{
        let mut cx = futures_test::task::noop_context();
        let mut buf = vec![0; $buf_size];
        match std::pin::Pin::new(&mut $reader).poll_read(&mut cx, &mut buf) {
            futures::task::Poll::Ready(_) => {
                panic!("assertion failed: reader is not pending");
            }
            futures::task::Poll::Pending => {}
        }
    }};
}

pub use assert_reader_pending;

#[macro_export]
macro_rules! assert_reader_ready {
    ($reader:expr) => {{
        assert_reader_ready!($reader, 100000)
    }};
    ($reader:expr, $buf_size:expr) => {{
        let mut cx = futures_test::task::noop_context();
        let mut buf = vec![0; $buf_size];
        match std::pin::Pin::new(&mut $reader).poll_read(&mut cx, &mut buf) {
            Poll::Ready(res) => res.map(|n| {
                buf.truncate(n);
                buf
            }),
            Poll::Pending => {
                panic!("assertion failed: reader is not ready");
            }
        }
    }};
}

pub use assert_reader_ready;

pub fn create_cipher() -> Cipher {
    Cipher::with_keys(
        [
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1,
        ],
        [
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2,
        ],
        [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3],
    )
}
