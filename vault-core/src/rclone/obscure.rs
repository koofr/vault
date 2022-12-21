/// Based on rclone implementation
/// https://github.com/rclone/rclone/blob/7be9855a706d1e09504f17949a90c54cd56fb2a5/fs/config/obscure/obscure.go
use aes::{
    self,
    cipher::{generic_array::GenericArray, FromBlockCipher, StreamCipher},
    NewBlockCipher,
};
use data_encoding::BASE64URL_NOPAD;
use rand_core::{OsRng, RngCore};
use thiserror::Error;

const CRYPT_KEY: [u8; 32] = [
    0x9c, 0x93, 0x5b, 0x48, 0x73, 0x0a, 0x55, 0x4d, 0x6b, 0xfd, 0x7c, 0x63, 0xc8, 0x86, 0xa9, 0x2b,
    0xd3, 0x90, 0x19, 0x8e, 0xb8, 0x12, 0x8a, 0xfb, 0xf4, 0xde, 0x16, 0x2b, 0x8b, 0x95, 0xf6, 0x38,
];

const AES_BLOCK_SIZE: usize = 16;

#[derive(Error, Debug)]
#[error("obscure error: {0}")]
pub struct ObscureError(String);

/// crypt transforms the buffer using iv under AES-CTR.
///
/// Note encryption and decryption are the same operation.
pub fn crypt(buf: &mut [u8], iv: &[u8]) -> Result<(), ObscureError> {
    let block_cipher = aes::Aes256::new_from_slice(&CRYPT_KEY).unwrap();
    let nonce = GenericArray::from_slice(&iv);
    let mut stream_cipher = aes::Aes256Ctr::from_block_cipher(block_cipher, &nonce);

    stream_cipher.try_apply_keystream(buf).map_err(|e| {
        ObscureError(format!(
            "reveal failed when revealing password - is it obscured? {}",
            e.to_string()
        ))
    })
}

// Obscure a value
//
// This is done by encrypting with AES-CTR
pub fn obscure(plaintext_password: &str) -> Result<String, ObscureError> {
    let mut iv = vec![0; AES_BLOCK_SIZE];

    (&mut OsRng)
        .try_fill_bytes(&mut iv)
        .map_err(|_| ObscureError(String::from("read iv failed")))?;

    let mut buf = plaintext_password.as_bytes().to_vec();

    crypt(&mut buf, &iv)?;

    let mut ciphertext = Vec::with_capacity(AES_BLOCK_SIZE + buf.len());
    ciphertext.extend_from_slice(&iv);
    ciphertext.extend_from_slice(&buf);

    Ok(BASE64URL_NOPAD.encode(&ciphertext))
}

/// Reveal an obscured value
pub fn reveal(encrypted_password: &str) -> Result<String, ObscureError> {
    let ciphertext = BASE64URL_NOPAD
        .decode(encrypted_password.as_bytes())
        .map_err(|e| {
            ObscureError(format!(
                "base64 decode failed when revealing password - is it obscured? {}",
                e.to_string()
            ))
        })?;

    if ciphertext.len() < AES_BLOCK_SIZE {
        return Err(ObscureError(format!(
            "input too short when revealing password - is it obscured?",
        )));
    }

    let iv = &ciphertext[..AES_BLOCK_SIZE];
    let mut buf = ciphertext[AES_BLOCK_SIZE..].to_vec();

    crypt(&mut buf, &iv)?;

    String::from_utf8(buf.to_vec()).map_err(|e| {
        ObscureError(format!(
            "utf8 parse failed when revealing password - is it obscured? {}",
            e.to_string()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{obscure, reveal};

    #[test]
    fn test_obscure() {
        assert_eq!(
            reveal(&obscure("obscuretest").unwrap()).unwrap(),
            String::from("obscuretest")
        );
    }

    #[test]
    fn test_reveal() {
        assert_eq!(
            reveal("PC0KdJEa4AhwNuQ_DuBli0xD4KTuj-l6HU1s").unwrap(),
            String::from("obscuretest")
        );
    }
}
