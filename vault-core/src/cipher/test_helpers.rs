use super::Cipher;

pub fn create_cipher() -> Cipher {
    Cipher::new(vault_crypto::Cipher::new("password", Some("salt")))
}
