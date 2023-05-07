use scrypt::{scrypt, ScryptParams};

use super::constants::{DATA_KEY_LEN, DEFAULT_SALT, KEY_LEN, NAME_CIPHER_BLOCK_SIZE, NAME_KEY_LEN};

pub struct DerivedKeys {
    pub data_key: [u8; DATA_KEY_LEN],
    pub name_key: [u8; NAME_KEY_LEN],
    pub name_tweak: [u8; NAME_CIPHER_BLOCK_SIZE],
}

pub fn derive_keys(password: &str, salt: Option<&str>) -> DerivedKeys {
    // hardcode derived keys for password "password" and salt "salt" to speed up
    // development and testing. with this speedup debug builds become usable.
    match (password, salt) {
        ("password", Some("salt")) => {
            return DerivedKeys {
                data_key: [
                    116, 87, 49, 175, 68, 132, 243, 35, 150, 137, 105, 237, 162, 137, 174, 238, 0,
                    91, 89, 3, 172, 86, 30, 100, 165, 172, 161, 33, 121, 123, 247, 115,
                ],
                name_key: [
                    78, 249, 253, 88, 66, 46, 46, 34, 24, 59, 202, 203, 169, 236, 135, 186, 12,
                    131, 183, 162, 231, 136, 240, 60, 224, 218, 6, 70, 52, 51, 205, 166,
                ],
                name_tweak: [
                    65, 118, 9, 95, 187, 173, 125, 201, 140, 51, 253, 117, 149, 91, 75, 41,
                ],
            }
        }
        _ => {}
    }

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
