/// Based on rclone implementation
/// https://github.com/rclone/rclone/blob/7be9855a706d1e09504f17949a90c54cd56fb2a5/backend/crypt/cipher.go
use xsalsa20poly1305::{
    aead::{AeadInPlace, KeyInit},
    XSalsa20Poly1305,
};

use super::{
    constants::{BLOCK_DATA_SIZE, BLOCK_HEADER_SIZE, BLOCK_SIZE, FILE_HEADER_SIZE},
    errors::DecryptSizeError,
    nonce::Nonce,
    CipherError,
};

pub fn get_data_cipher(data_key: &[u8]) -> XSalsa20Poly1305 {
    XSalsa20Poly1305::new(data_key.into())
}

pub fn encrypt_block(
    data_cipher: &XSalsa20Poly1305,
    nonce: &Nonce,
    plaintext: &[u8],
) -> Result<Vec<u8>, CipherError> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(plaintext);

    data_cipher
        .encrypt_in_place(nonce.as_nonce(), b"", &mut buffer)
        .map_err(|_| CipherError::EncryptionError)?;

    Ok(buffer)
}

/// encrypted_size calculates the size of the data when encrypted
pub fn encrypted_size(size: i64) -> i64 {
    let blocks = size / BLOCK_DATA_SIZE as i64;
    let residue = size % BLOCK_DATA_SIZE as i64;

    let mut encrypted_size = FILE_HEADER_SIZE as i64 + blocks * BLOCK_SIZE as i64;

    if residue != 0 {
        encrypted_size = encrypted_size + BLOCK_HEADER_SIZE as i64 + residue;
    }

    encrypted_size
}

pub fn decrypt_block(
    data_cipher: &XSalsa20Poly1305,
    nonce: &Nonce,
    ciphertext: &[u8],
) -> Result<Vec<u8>, CipherError> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(ciphertext);

    data_cipher
        .decrypt_in_place(nonce.as_nonce(), b"", &mut buffer)
        .map_err(|_| CipherError::DecryptionError)?;

    Ok(buffer)
}

/// decrypt_size calculates the size of the data when decrypted
pub fn decrypt_size(size: i64) -> Result<i64, DecryptSizeError> {
    let size = size - FILE_HEADER_SIZE as i64;
    if size < 0 {
        return Err(DecryptSizeError::EncryptedFileTooShort);
    }
    let blocks = size / BLOCK_SIZE as i64;
    let mut residue = size % BLOCK_SIZE as i64;
    let mut decrypted_size = blocks * BLOCK_DATA_SIZE as i64;
    if residue != 0 {
        residue = residue - BLOCK_HEADER_SIZE as i64;
        if residue < 0 {
            return Err(DecryptSizeError::EncryptedFileBadHeader);
        }
    }
    decrypted_size = decrypted_size + residue;

    Ok(decrypted_size)
}
