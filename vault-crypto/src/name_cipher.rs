use aes::Aes256;
use data_encoding::BASE32HEX_NOPAD;
use eme_mode::{block_modes::BlockMode, block_padding::Pkcs7, Eme};

use super::{constants::NAME_CIPHER_BLOCK_SIZE, errors::DecryptFilenameError};

type Aes256Eme = Eme<Aes256, Pkcs7>;

pub fn get_name_cipher(name_key: &[u8], name_tweak: &[u8]) -> Aes256Eme {
    Aes256Eme::new_from_slices(&name_key, &name_tweak).unwrap()
}

pub fn encrypt_filename(name_cipher: Aes256Eme, plaintext: &str) -> String {
    let plaintext_bytes = plaintext.as_bytes();

    let pos = plaintext_bytes.len();
    let padding = NAME_CIPHER_BLOCK_SIZE - (pos % NAME_CIPHER_BLOCK_SIZE);

    let mut buffer = vec![0; pos + padding];
    buffer[..pos].copy_from_slice(plaintext_bytes);

    let encrypted = name_cipher.encrypt(&mut buffer, pos).unwrap();

    BASE32HEX_NOPAD.encode(&encrypted).to_lowercase()
}

pub fn encrypt_path(name_cipher: Aes256Eme, plaintext: &str) -> String {
    match plaintext {
        "/" => plaintext.to_owned(),
        _ => {
            let parts: Vec<&str> = plaintext.split("/").skip(1).collect();
            let mut encrypted_parts: Vec<String> = Vec::with_capacity(parts.len() + 1);
            encrypted_parts.push(String::from(""));
            for part in parts {
                let encrypted_part = encrypt_filename(name_cipher.clone(), &part);
                encrypted_parts.push(encrypted_part);
            }
            encrypted_parts.join("/")
        }
    }
}

pub fn decrypt_filename(
    name_cipher: Aes256Eme,
    ciphertext: &str,
) -> Result<String, DecryptFilenameError> {
    if ciphertext.is_empty() {
        return Ok(String::from(""));
    }

    let mut name_encrypted_buf = BASE32HEX_NOPAD
        .decode(ciphertext.to_uppercase().as_bytes())
        .map_err(|e| DecryptFilenameError::DecodeError(e.to_string()))?;

    let decrypted = name_cipher
        .decrypt(name_encrypted_buf.as_mut_slice().into())
        .map_err(|_| DecryptFilenameError::DecryptError)?;

    String::from_utf8(decrypted.to_vec())
        .map_err(|e| DecryptFilenameError::UnicodeError(e.to_string()))
}

pub fn decrypt_path(
    name_cipher: Aes256Eme,
    ciphertext: &str,
) -> Result<String, DecryptFilenameError> {
    match ciphertext {
        "/" => Ok(ciphertext.to_owned()),
        _ => {
            let parts: Vec<&str> = ciphertext.split("/").skip(1).collect();
            let mut decrypted_parts: Vec<String> = Vec::with_capacity(parts.len() + 1);
            decrypted_parts.push(String::from(""));
            for part in parts {
                let decrypted_part = decrypt_filename(name_cipher.clone(), &part)?;
                decrypted_parts.push(decrypted_part);
            }
            Ok(decrypted_parts.join("/"))
        }
    }
}
