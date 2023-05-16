use core::mem;
use futures::{
    ready,
    task::{Context, Poll},
    AsyncRead,
};
use pin_project_lite::pin_project;
use std::{cmp, io::Result, pin::Pin, sync::Arc};
use xsalsa20poly1305::XSalsa20Poly1305;

use super::{
    constants::{BLOCK_HEADER_SIZE, BLOCK_SIZE, FILE_MAGIC, FILE_MAGIC_SIZE, FILE_NONCE_SIZE},
    data_cipher::decrypt_block,
    nonce::Nonce,
    CipherError,
};

#[derive(Debug)]
pub enum DecryptReaderState {
    ReadingMagic {
        buffer: [u8; FILE_MAGIC_SIZE],
        pos: usize,
    },
    ReadingNonce {
        buffer: [u8; FILE_NONCE_SIZE],
        pos: usize,
    },
    ReadingCiphertext {
        nonce: Nonce,
        buffer: [u8; BLOCK_SIZE],
        pos: usize,
    },
    WritingPlaintext {
        nonce: Nonce,
        buffer: Vec<u8>,
        pos: usize,
    },
}

pin_project! {
    pub struct DecryptReader<R> {
        #[pin]
        inner: R,
        state: DecryptReaderState,
        data_cipher: Arc<XSalsa20Poly1305>
    }
}

impl<R> DecryptReader<R> {
    pub fn new(inner: R, data_cipher: Arc<XSalsa20Poly1305>) -> Self {
        Self {
            inner,
            state: DecryptReaderState::ReadingMagic {
                buffer: [0; FILE_MAGIC_SIZE],
                pos: 0,
            },
            data_cipher,
        }
    }
}

impl<R: AsyncRead> AsyncRead for DecryptReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let mut this = self.project();

        loop {
            match this.state {
                DecryptReaderState::ReadingMagic { buffer, pos } => {
                    let n = ready!(this.inner.as_mut().poll_read(cx, &mut buffer[*pos..]))?;

                    if n == 0 {
                        return Poll::Ready(Err(CipherError::EncryptedFileTooShort.into()));
                    }

                    *pos += n;

                    if *pos == FILE_MAGIC_SIZE {
                        let magic = &buffer[0..FILE_MAGIC_SIZE];

                        if magic != FILE_MAGIC {
                            return Poll::Ready(Err(CipherError::EncryptedBadMagic.into()));
                        }

                        *this.state = DecryptReaderState::ReadingNonce {
                            buffer: [0; FILE_NONCE_SIZE],
                            pos: 0,
                        };
                    }
                }
                DecryptReaderState::ReadingNonce { buffer, pos } => {
                    let n = ready!(this.inner.as_mut().poll_read(cx, &mut buffer[*pos..]))?;

                    if n == 0 {
                        return Poll::Ready(Err(CipherError::EncryptedFileTooShort.into()));
                    }

                    *pos += n;

                    if *pos == FILE_NONCE_SIZE {
                        let nonce = &buffer[..FILE_NONCE_SIZE];

                        *this.state = DecryptReaderState::ReadingCiphertext {
                            nonce: Nonce::new(nonce.try_into().unwrap()),
                            buffer: [0; BLOCK_SIZE],
                            pos: 0,
                        };
                    }
                }
                DecryptReaderState::ReadingCiphertext { nonce, buffer, pos } => {
                    let n = ready!(this.inner.as_mut().poll_read(cx, &mut buffer[*pos..]))?;

                    if n == 0 && *pos == 0 {
                        return Poll::Ready(Ok(0));
                    }

                    *pos += n;

                    if n == 0 || *pos == BLOCK_SIZE {
                        if *pos <= BLOCK_HEADER_SIZE {
                            return Poll::Ready(Err(CipherError::EncryptedFileBadHeader.into()));
                        }

                        let decrypted =
                            match decrypt_block(this.data_cipher, &nonce, &buffer[..*pos]) {
                                Ok(decrypted) => decrypted,
                                Err(e) => return Poll::Ready(Err(e.into())),
                            };

                        nonce.increment();

                        *this.state = DecryptReaderState::WritingPlaintext {
                            nonce: mem::take(nonce),
                            buffer: decrypted,
                            pos: 0,
                        };
                    }
                }
                DecryptReaderState::WritingPlaintext { nonce, buffer, pos } => {
                    let n = cmp::min(buf.len(), buffer.len() - *pos);

                    buf[..n].copy_from_slice(&buffer[*pos..*pos + n]);

                    *pos += n;

                    if *pos == buffer.len() {
                        *this.state = DecryptReaderState::ReadingCiphertext {
                            nonce: mem::take(nonce),
                            buffer: [0; BLOCK_SIZE],
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

    use futures::{channel::mpsc, stream::TryStreamExt, AsyncRead};
    use xsalsa20poly1305::XSalsa20Poly1305;

    use crate::cipher::{
        constants::FILE_MAGIC,
        data_cipher::{encrypt_block, get_data_cipher},
        nonce::Nonce,
        test_helpers::{assert_reader_pending, assert_reader_ready},
    };

    use super::DecryptReader;

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
    fn test_decrypt_reader() {
        let data_cipher = get_dummy_data_cipher();
        let nonce = get_dummy_nonce();

        let (tx, rx) = mpsc::unbounded::<Result<Vec<u8>>>();
        let reader = rx.into_async_read();

        let mut r = DecryptReader::new(reader, data_cipher.clone());

        assert_reader_pending!(r);

        tx.unbounded_send(Ok(FILE_MAGIC[..4].to_vec())).unwrap();

        assert_reader_pending!(r);

        tx.unbounded_send(Ok(FILE_MAGIC[4..].to_vec())).unwrap();

        assert_reader_pending!(r);

        tx.unbounded_send(Ok(nonce.as_slice()[..12].to_vec()))
            .unwrap();

        assert_reader_pending!(r);

        tx.unbounded_send(Ok(nonce.as_slice()[12..].to_vec()))
            .unwrap();

        assert_reader_pending!(r);

        tx.unbounded_send(Ok(encrypt_block(&data_cipher, &nonce, b"test").unwrap()))
            .unwrap();

        assert_reader_pending!(r);

        tx.close_channel();

        let res1 = assert_reader_ready!(r, 3).unwrap();
        assert_eq!(res1.len(), 3);
        let res2 = assert_reader_ready!(r, 3).unwrap();
        assert_eq!(res2.len(), 1);
        let res = concat_vecs(&mut [res1, res2]);

        assert_eq!(res, b"test");
    }
}
