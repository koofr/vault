use std::fmt::Debug;

use crate::{
    types::{DecryptedName, DecryptedPath, EncryptedName, EncryptedPath},
    utils::name_utils,
};

use super::errors::DecryptFilenameError;

pub struct Cipher {
    cipher: vault_crypto::Cipher,
}

impl Debug for Cipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cipher").finish()
    }
}

impl Cipher {
    pub fn new(cipher: vault_crypto::Cipher) -> Self {
        Self { cipher }
    }

    pub fn encrypt_filename(&self, plaintext: &DecryptedName) -> EncryptedName {
        EncryptedName(self.cipher.encrypt_filename(&plaintext.0))
    }

    pub fn encrypt_path(&self, plaintext: &DecryptedPath) -> EncryptedPath {
        EncryptedPath(self.cipher.encrypt_path(&plaintext.0))
    }

    pub fn decrypt_filename(
        &self,
        ciphertext: &EncryptedName,
    ) -> Result<DecryptedName, DecryptFilenameError> {
        self.cipher
            .decrypt_filename(&ciphertext.0)
            .map(DecryptedName)
            .map_err(DecryptFilenameError::DecryptFilenameError)
            .and_then(|name| match name_utils::validate_name(&name.0) {
                Ok(()) => Ok(name),
                Err(err) => Err(DecryptFilenameError::InvalidNameError(err)),
            })
    }

    pub fn decrypt_path(
        &self,
        ciphertext: &EncryptedPath,
    ) -> Result<DecryptedPath, DecryptFilenameError> {
        self.cipher
            .decrypt_path(&ciphertext.0)
            .map(DecryptedPath)
            .map_err(Into::into)
            .and_then(|path| {
                if path.0 != "/" {
                    for name in path.0.split("/").skip(1) {
                        if let Err(err) = name_utils::validate_name(&name) {
                            return Err(DecryptFilenameError::InvalidNameError(err));
                        }
                    }
                }

                Ok(path)
            })
    }

    pub fn encrypt_reader_async<R>(
        &self,
        reader: R,
    ) -> vault_crypto::encrypt_reader::AsyncEncryptReader<R> {
        self.cipher.encrypt_reader_async(reader)
    }

    pub fn encrypt_reader_sync<R>(
        &self,
        reader: R,
    ) -> vault_crypto::encrypt_reader::SyncEncryptReader<R> {
        self.cipher.encrypt_reader_sync(reader)
    }

    pub fn encrypt_data(&self, data: &[u8], out: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        self.cipher.encrypt_data(data, out)
    }

    pub fn decrypt_reader_async<R>(
        &self,
        reader: R,
    ) -> vault_crypto::decrypt_reader::AsyncDecryptReader<R> {
        self.cipher.decrypt_reader_async(reader)
    }

    pub fn decrypt_reader_sync<R>(
        &self,
        reader: R,
    ) -> vault_crypto::decrypt_reader::SyncDecryptReader<R> {
        self.cipher.decrypt_reader_sync(reader)
    }

    pub fn decrypt_data(&self, data: &[u8], out: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        self.cipher.decrypt_data(data, out)
    }
}
