use data_encoding::BASE64URL_NOPAD;
use uuid::Uuid;

use crate::{cipher::Cipher, types::RemoteName};

pub async fn generate_password_validator(cipher: &Cipher) -> (String, String) {
    let password_validator = Uuid::new_v4().to_string();

    let mut password_validator_encrypted_bytes = Vec::new();

    cipher
        .encrypt_data(
            password_validator.as_bytes(),
            &mut password_validator_encrypted_bytes,
        )
        .await
        .unwrap();

    let password_validator_encrypted = format!(
        "v2:{}",
        BASE64URL_NOPAD.encode(&password_validator_encrypted_bytes)
    );

    (password_validator, password_validator_encrypted)
}

pub async fn check_password_validator(
    cipher: &Cipher,
    password_validator: &str,
    password_validator_encrypted: &str,
) -> bool {
    if password_validator_encrypted.starts_with("v2:") {
        check_password_validator_v2(cipher, password_validator, password_validator_encrypted).await
    } else {
        check_password_validator_v1(cipher, password_validator, password_validator_encrypted)
    }
}

pub fn check_password_validator_v1(
    cipher: &Cipher,
    password_validator: &str,
    password_validator_encrypted: &str,
) -> bool {
    cipher
        .decrypt_filename(&RemoteName(password_validator_encrypted.to_owned()))
        .ok()
        .filter(|password_validator_decrypted| password_validator_decrypted.0 == password_validator)
        .is_some()
}

pub async fn check_password_validator_v2(
    cipher: &Cipher,
    password_validator: &str,
    password_validator_encrypted: &str,
) -> bool {
    if !password_validator_encrypted.starts_with("v2:") {
        return false;
    }

    let password_validator_encrypted_bytes =
        match BASE64URL_NOPAD.decode(password_validator_encrypted[3..].as_bytes()) {
            Ok(x) => x,
            _ => return false,
        };

    let mut password_validator_decrypted_bytes = Vec::new();

    if cipher
        .decrypt_data(
            &password_validator_encrypted_bytes,
            &mut password_validator_decrypted_bytes,
        )
        .await
        .is_err()
    {
        return false;
    }

    let password_validator_decrypted =
        match std::str::from_utf8(&password_validator_decrypted_bytes) {
            Ok(x) => x,
            _ => return false,
        };

    password_validator_decrypted == password_validator
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use crate::{cipher::Cipher, repos::password_validator::check_password_validator};

    use super::generate_password_validator;

    #[test]
    fn test_generate_password_validator() {
        block_on(async {
            let cipher = Cipher::new("testpassword", None);

            let (password_validator, password_validator_encrypted) =
                generate_password_validator(&cipher).await;

            assert_eq!(password_validator.len(), 36);
            assert!(password_validator_encrypted.starts_with("v2:"));

            println!("{}", password_validator);
            println!("{}", password_validator_encrypted);

            assert!(
                check_password_validator(
                    &cipher,
                    &password_validator,
                    &password_validator_encrypted
                )
                .await
            )
        });
    }

    #[test]
    fn test_check_password_validator_v1() {
        block_on(async {
            let cipher = Cipher::new("testpassword", None);

            let password_validator = "d645d972-d7f4-4577-bec6-b52652c025c9";
            let password_validator_encrypted =
                "lb96gl718rmaq911ehuan90tu6ta5sg6k38fpd4hsj91p4h0tvd5ouk37663f9jacrl9eaq4depri";

            assert!(
                check_password_validator(
                    &cipher,
                    &password_validator,
                    &password_validator_encrypted
                )
                .await
            )
        })
    }

    #[test]
    fn test_check_password_validator_v2() {
        block_on(async {
            let cipher = Cipher::new("testpassword", None);

            let password_validator = "508ddd3f-f18e-4514-932b-b2c1f0c8b291";
            let password_validator_encrypted =
                "v2:UkNMT05FAAA-YjvGKKxTpiFekFYVMNO2UnG2u-Z16MMHAB-ipQYycVTmPSNk0mbnYeZrZ2I-Kh0lTmh4Kt2UxhdYWEXd9YQvyODrWMWWHZaLhL7e";

            assert!(
                check_password_validator(
                    &cipher,
                    &password_validator,
                    &password_validator_encrypted
                )
                .await
            )
        })
    }
}
