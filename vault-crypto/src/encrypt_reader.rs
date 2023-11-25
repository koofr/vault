use core::mem;
use std::{
    cmp,
    io::{Read, Result},
    pin::Pin,
    sync::Arc,
};

use futures::{
    ready,
    task::{Context, Poll},
    AsyncRead,
};
use pin_project_lite::pin_project;
use xsalsa20poly1305::XSalsa20Poly1305;

use super::{
    constants::{BLOCK_DATA_SIZE, FILE_MAGIC, FILE_MAGIC_SIZE, FILE_NONCE_SIZE},
    data_cipher::encrypt_block,
    nonce::Nonce,
};

#[derive(Debug)]
pub enum EncryptReaderState {
    WritingMagic {
        nonce: Nonce,
        pos: usize,
    },
    WritingNonce {
        nonce: Nonce,
        pos: usize,
    },
    ReadingPlaintext {
        nonce: Nonce,
        buffer: Vec<u8>,
        pos: usize,
    },
    WritingCiphertext {
        nonce: Nonce,
        buffer: Vec<u8>,
        pos: usize,
    },
}

pub struct SyncEncryptReader<R> {
    inner: R,
    state: EncryptReaderState,
    data_cipher: Arc<XSalsa20Poly1305>,
}

impl<R> SyncEncryptReader<R> {
    pub fn new(inner: R, data_cipher: Arc<XSalsa20Poly1305>, nonce: Nonce) -> Self {
        Self {
            inner,
            state: EncryptReaderState::WritingMagic { nonce, pos: 0 },
            data_cipher,
        }
    }
}

impl<R: Read> Read for SyncEncryptReader<R> {
    fn read(self: &mut Self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match &mut self.state {
                EncryptReaderState::WritingMagic { nonce, pos } => {
                    let n = cmp::min(buf.len(), FILE_MAGIC_SIZE - *pos);

                    buf[..n].copy_from_slice(&FILE_MAGIC[*pos..*pos + n]);

                    *pos += n;

                    if *pos == FILE_MAGIC_SIZE {
                        self.state = EncryptReaderState::WritingNonce {
                            nonce: mem::take(nonce),
                            pos: 0,
                        };
                    }

                    return Ok(n);
                }
                EncryptReaderState::WritingNonce { nonce, pos } => {
                    let n = cmp::min(buf.len(), FILE_NONCE_SIZE - *pos);

                    buf[..n].copy_from_slice(&nonce.as_slice()[*pos..*pos + n]);

                    *pos += n;

                    if *pos == FILE_NONCE_SIZE {
                        self.state = EncryptReaderState::ReadingPlaintext {
                            nonce: mem::take(nonce),
                            buffer: vec![0; BLOCK_DATA_SIZE],
                            pos: 0,
                        };
                    }

                    return Ok(n);
                }
                EncryptReaderState::ReadingPlaintext { nonce, buffer, pos } => {
                    let n = self.inner.read(&mut buffer[*pos..])?;

                    if n == 0 && *pos == 0 {
                        return Ok(0);
                    }

                    *pos += n;

                    if n == 0 || *pos == BLOCK_DATA_SIZE {
                        let encrypted =
                            match encrypt_block(&self.data_cipher, &nonce, &buffer[..*pos]) {
                                Ok(encrypted) => encrypted,
                                Err(e) => return Err(e.into()),
                            };

                        nonce.increment();

                        self.state = EncryptReaderState::WritingCiphertext {
                            nonce: mem::take(nonce),
                            buffer: encrypted,
                            pos: 0,
                        };
                    }
                }
                EncryptReaderState::WritingCiphertext { nonce, buffer, pos } => {
                    let n = cmp::min(buf.len(), buffer.len() - *pos);

                    buf[..n].copy_from_slice(&buffer[*pos..*pos + n]);

                    *pos += n;

                    if *pos == buffer.len() {
                        self.state = EncryptReaderState::ReadingPlaintext {
                            nonce: mem::take(nonce),
                            buffer: vec![0; BLOCK_DATA_SIZE],
                            pos: 0,
                        };
                    }

                    return Ok(n);
                }
            }
        }
    }
}

pin_project! {
    pub struct AsyncEncryptReader<R> {
        #[pin]
        inner: R,
        state: EncryptReaderState,
        data_cipher: Arc<XSalsa20Poly1305>
    }
}

impl<R> AsyncEncryptReader<R> {
    pub fn new(inner: R, data_cipher: Arc<XSalsa20Poly1305>, nonce: Nonce) -> Self {
        Self {
            inner,
            state: EncryptReaderState::WritingMagic { nonce, pos: 0 },
            data_cipher,
        }
    }
}

impl<R: AsyncRead> AsyncRead for AsyncEncryptReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let mut this = self.project();

        loop {
            match this.state {
                EncryptReaderState::WritingMagic { nonce, pos } => {
                    let n = cmp::min(buf.len(), FILE_MAGIC_SIZE - *pos);

                    buf[..n].copy_from_slice(&FILE_MAGIC[*pos..*pos + n]);

                    *pos += n;

                    if *pos == FILE_MAGIC_SIZE {
                        *this.state = EncryptReaderState::WritingNonce {
                            nonce: mem::take(nonce),
                            pos: 0,
                        };
                    }

                    return Poll::Ready(Ok(n));
                }
                EncryptReaderState::WritingNonce { nonce, pos } => {
                    let n = cmp::min(buf.len(), FILE_NONCE_SIZE - *pos);

                    buf[..n].copy_from_slice(&nonce.as_slice()[*pos..*pos + n]);

                    *pos += n;

                    if *pos == FILE_NONCE_SIZE {
                        *this.state = EncryptReaderState::ReadingPlaintext {
                            nonce: mem::take(nonce),
                            buffer: vec![0; BLOCK_DATA_SIZE],
                            pos: 0,
                        };
                    }

                    return Poll::Ready(Ok(n));
                }
                EncryptReaderState::ReadingPlaintext { nonce, buffer, pos } => {
                    let n = ready!(this.inner.as_mut().poll_read(cx, &mut buffer[*pos..]))?;

                    if n == 0 && *pos == 0 {
                        return Poll::Ready(Ok(0));
                    }

                    *pos += n;

                    if n == 0 || *pos == BLOCK_DATA_SIZE {
                        let encrypted =
                            match encrypt_block(this.data_cipher, &nonce, &buffer[..*pos]) {
                                Ok(encrypted) => encrypted,
                                Err(e) => return Poll::Ready(Err(e.into())),
                            };

                        nonce.increment();

                        *this.state = EncryptReaderState::WritingCiphertext {
                            nonce: mem::take(nonce),
                            buffer: encrypted,
                            pos: 0,
                        };
                    }
                }
                EncryptReaderState::WritingCiphertext { nonce, buffer, pos } => {
                    let n = cmp::min(buf.len(), buffer.len() - *pos);

                    buf[..n].copy_from_slice(&buffer[*pos..*pos + n]);

                    *pos += n;

                    if *pos == buffer.len() {
                        *this.state = EncryptReaderState::ReadingPlaintext {
                            nonce: mem::take(nonce),
                            buffer: vec![0; BLOCK_DATA_SIZE],
                            pos: 0,
                        };
                    }

                    return Poll::Ready(Ok(n));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Result, sync::Arc, task::Poll};

    use futures::{stream::TryStreamExt, AsyncRead};
    use xsalsa20poly1305::XSalsa20Poly1305;

    use crate::{
        constants::FILE_MAGIC,
        data_cipher::{decrypt_block, get_data_cipher},
        encrypt_reader::SyncEncryptReader,
        nonce::Nonce,
        test_helpers::{assert_reader_pending, assert_reader_ready},
    };

    use super::AsyncEncryptReader;

    fn get_dummy_data_cipher() -> Arc<XSalsa20Poly1305> {
        let data_key = [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ];

        Arc::new(get_data_cipher(&data_key))
    }

    fn get_dummy_nonce() -> Nonce {
        Nonce::new(&[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ])
    }

    fn concat_vecs(vecs: &mut [Vec<u8>]) -> Vec<u8> {
        let mut res = vec![];
        for vec in vecs {
            res.append(vec);
        }
        res
    }

    #[test]
    fn test_sync_encrypt_reader() {
        let data_cipher = get_dummy_data_cipher();
        let nonce = get_dummy_nonce();

        let reader = std::io::Cursor::new(b"test".to_vec());

        let mut r = SyncEncryptReader::new(reader, data_cipher.clone(), nonce.clone());

        fn read(r: &mut impl std::io::Read, n: usize) -> Result<Vec<u8>> {
            let mut buf = vec![0; n];
            r.read(&mut buf).map(|n| {
                buf.truncate(n);
                buf
            })
        }

        let res1 = read(&mut r, 7).unwrap();
        assert_eq!(res1.len(), 7);
        let res2 = read(&mut r, 7).unwrap();
        assert_eq!(res2.len(), 1);
        let res = concat_vecs(&mut [res1, res2]);

        assert_eq!(res, FILE_MAGIC);

        let res1 = read(&mut r, 23).unwrap();
        assert_eq!(res1.len(), 23);
        let res2 = read(&mut r, 23).unwrap();
        assert_eq!(res2.len(), 1);
        let res = concat_vecs(&mut [res1, res2]);

        assert_eq!(res, nonce.as_slice().to_vec());

        let res1 = read(&mut r, 18).unwrap();
        assert_eq!(res1.len(), 18);
        let res2 = read(&mut r, 18).unwrap();
        assert_eq!(res2.len(), 2);
        let res = concat_vecs(&mut [res1, res2]);

        let plaintext = decrypt_block(&data_cipher, &nonce, &res).unwrap();

        assert_eq!(plaintext, b"test");

        let res = read(&mut r, 10000);
        assert_eq!(res.unwrap().len(), 0);
    }

    #[test]
    fn test_async_encrypt_reader() {
        let data_cipher = get_dummy_data_cipher();
        let nonce = get_dummy_nonce();

        let (tx, rx) = futures::channel::mpsc::unbounded::<Result<Vec<u8>>>();
        let reader = rx.into_async_read();

        let mut r = AsyncEncryptReader::new(reader, data_cipher.clone(), nonce.clone());

        let res1 = assert_reader_ready!(r, 7).unwrap();
        assert_eq!(res1.len(), 7);
        let res2 = assert_reader_ready!(r, 7).unwrap();
        assert_eq!(res2.len(), 1);
        let res = concat_vecs(&mut [res1, res2]);

        assert_eq!(res, FILE_MAGIC);

        let res1 = assert_reader_ready!(r, 23).unwrap();
        assert_eq!(res1.len(), 23);
        let res2 = assert_reader_ready!(r, 23).unwrap();
        assert_eq!(res2.len(), 1);
        let res = concat_vecs(&mut [res1, res2]);

        assert_eq!(res, nonce.as_slice().to_vec());

        assert_reader_pending!(r);

        tx.unbounded_send(Ok(b"test".to_vec())).unwrap();

        assert_reader_pending!(r);

        tx.close_channel();

        let res1 = assert_reader_ready!(r, 18).unwrap();
        assert_eq!(res1.len(), 18);
        let res2 = assert_reader_ready!(r, 18).unwrap();
        assert_eq!(res2.len(), 2);
        let res = concat_vecs(&mut [res1, res2]);

        let plaintext = decrypt_block(&data_cipher, &nonce, &res).unwrap();

        assert_eq!(plaintext, b"test");

        let res = assert_reader_ready!(r);

        assert_eq!(res.unwrap().len(), 0);
    }
}
