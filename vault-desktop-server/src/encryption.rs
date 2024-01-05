use rand_core::{OsRng, RngCore};
use thiserror::Error;

use vault_crypto::{
    constants::{DATA_KEY_LEN, FILE_NONCE_SIZE},
    data_cipher::{decrypt_block, encrypt_block, get_data_cipher, XSalsa20Poly1305},
    nonce::Nonce,
    CipherError,
};

#[derive(Error, Debug, Clone)]
pub enum EncryptionError {
    #[error("base64 decode error: {0}")]
    Base64DecodeError(#[from] data_encoding::DecodeError),
    #[error("invalid nonce")]
    InvalidNonce,
    #[error("cipher error: {0}")]
    CipherError(#[from] CipherError),
}

#[derive(Clone)]
pub struct Encryption {
    cipher: XSalsa20Poly1305,
}

impl Encryption {
    pub fn new(cipher: XSalsa20Poly1305) -> Self {
        Self { cipher }
    }

    pub fn new_with_key_bytes(key_bytes: &[u8]) -> Self {
        let cipher = get_data_cipher(key_bytes);

        Self::new(cipher)
    }

    pub fn new_with_key_str(key_str: &str) -> Result<Self, data_encoding::DecodeError> {
        let key_bytes = data_encoding::BASE64.decode(key_str.as_bytes())?;

        Ok(Self::new_with_key_bytes(&key_bytes))
    }

    pub fn random() -> Result<(Self, String), EncryptionError> {
        let mut key: [u8; DATA_KEY_LEN] = [0; DATA_KEY_LEN];

        (&mut OsRng)
            .try_fill_bytes(&mut key)
            .map_err(|err| EncryptionError::CipherError(CipherError::from(err)))?;

        let key_str = data_encoding::BASE64.encode(&key[..]);

        Ok((Self::new_with_key_bytes(&key[..]), key_str))
    }

    pub fn encrypt(&self, decrypted: &[u8]) -> Result<String, EncryptionError> {
        let nonce = Nonce::new_random()?;

        let mut encrypted_data = encrypt_block(&self.cipher, &nonce, &decrypted).unwrap();

        let mut encrypted =
            Vec::with_capacity(vault_crypto::constants::FILE_NONCE_SIZE + encrypted_data.len());
        encrypted.extend_from_slice(nonce.as_slice());
        encrypted.append(&mut encrypted_data);

        Ok(data_encoding::BASE64.encode(&encrypted))
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        let encrypted = data_encoding::BASE64.decode(encrypted)?;

        let nonce = encrypted[0..FILE_NONCE_SIZE]
            .try_into()
            .map_err(|_| EncryptionError::InvalidNonce)?;
        let nonce = Nonce::new(nonce);

        let data = decrypt_block(&self.cipher, &nonce, &encrypted[FILE_NONCE_SIZE..])?;

        Ok(data)
    }
}
