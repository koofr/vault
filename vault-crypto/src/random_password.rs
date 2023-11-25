use data_encoding::BASE64URL_NOPAD;
use rand_core::{OsRng, RngCore};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
#[error("random password error: {0}")]
pub struct RandomPasswordError(String);

pub fn random_password(bits: usize) -> Result<String, RandomPasswordError> {
    let mut bytes = bits / 8;
    if bits % 8 != 0 {
        bytes += 1;
    }

    let mut password = vec![0; bytes];

    (&mut OsRng)
        .try_fill_bytes(&mut password)
        .map_err(|_| RandomPasswordError(String::from("password read failed")))?;

    Ok(BASE64URL_NOPAD.encode(&password))
}
