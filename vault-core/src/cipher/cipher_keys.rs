use scrypt::{scrypt, ScryptParams};

use super::constants::{DATA_KEY_LEN, DEFAULT_SALT, KEY_LEN, NAME_CIPHER_BLOCK_SIZE, NAME_KEY_LEN};

pub struct DerivedKeys {
    pub data_key: [u8; DATA_KEY_LEN],
    pub name_key: [u8; NAME_KEY_LEN],
    pub name_tweak: [u8; NAME_CIPHER_BLOCK_SIZE],
}

pub fn derive_keys(password: &str, salt: Option<&str>) -> DerivedKeys {
    let password_bytes = password.as_bytes();
    let salt_bytes = match &salt {
        Some(salt) => salt.as_bytes(),
        None => DEFAULT_SALT,
    };
    // Based on rclone implementation
    // https://github.com/rclone/rclone/blob/7be9855a706d1e09504f17949a90c54cd56fb2a5/backend/crypt/cipher.go#L219
    let log_n = 14; // log2 16384
    let scrypt_params = ScryptParams::new(log_n, 8, 1).unwrap();
    let mut scrypt_output: [u8; KEY_LEN] = [0; KEY_LEN];

    scrypt(
        password_bytes,
        salt_bytes,
        &scrypt_params,
        &mut scrypt_output,
    )
    .unwrap();

    DerivedKeys {
        data_key: scrypt_output[0..DATA_KEY_LEN].try_into().unwrap(),
        name_key: scrypt_output[DATA_KEY_LEN..DATA_KEY_LEN + NAME_KEY_LEN]
            .try_into()
            .unwrap(),
        name_tweak: scrypt_output[DATA_KEY_LEN + NAME_KEY_LEN..]
            .try_into()
            .unwrap(),
    }
}
