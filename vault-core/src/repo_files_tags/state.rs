use data_encoding::BASE64URL_NOPAD;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, sync::Arc};

use crate::cipher::Cipher;

use super::errors::{RepoFileTagsDecodeError, RepoFileTagsEncodeError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoFileTags {
    #[serde(
        default,
        rename = "eh",
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub encrypted_hash: Option<Vec<u8>>,
    #[serde(
        default,
        rename = "h",
        with = "serde_bytes",
        skip_serializing_if = "Option::is_none"
    )]
    pub hash: Option<Vec<u8>>,
    #[serde(flatten)]
    pub unknown: HashMap<String, rmpv::Value>,
}

impl RepoFileTags {
    pub fn encrypted_hash_hex(&self) -> Option<String> {
        self.encrypted_hash.as_deref().map(|hash| hex::encode(hash))
    }

    pub fn hash_hex(&self) -> Option<String> {
        self.hash.as_ref().map(hex::encode)
    }

    pub fn from_string(input: &str, cipher: &Cipher) -> Result<Self, RepoFileTagsDecodeError> {
        let encrypted = BASE64URL_NOPAD.decode(input.as_bytes())?;

        let encoded = cipher
            .decrypt_vec(&encrypted)
            .map_err(|err| RepoFileTagsDecodeError::DecryptError(Arc::new(err)))?;

        let tags = rmp_serde::from_slice(&encoded)
            .map_err(|err| RepoFileTagsDecodeError::RMPSerdeError(Arc::new(err)))?;

        Ok(tags)
    }

    pub fn to_string(&self, cipher: &Cipher) -> Result<String, RepoFileTagsEncodeError> {
        let encoded = rmp_serde::to_vec(self)
            .map_err(|err| RepoFileTagsEncodeError::RMPSerdeError(Arc::new(err)))?;

        let encrypted = cipher
            .encrypt_vec(&encoded)
            .map_err(|err| RepoFileTagsEncodeError::EncryptError(Arc::new(err)))?;

        Ok(BASE64URL_NOPAD.encode(&encrypted))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use data_encoding::BASE64URL_NOPAD;
    use similar_asserts::assert_eq;

    use crate::{
        cipher::test_helpers::create_cipher, repo_files_tags::errors::RepoFileTagsDecodeError,
        user_error::UserError,
    };

    use super::RepoFileTags;

    #[test]
    fn from_string() {
        let cipher = create_cipher();

        let encoded = rmp_serde::to_vec(&HashMap::from([
            (
                "eh".to_string(),
                rmpv::Value::Binary(vec![
                    10, 11, 64, 74, 230, 55, 68, 151, 148, 25, 24, 88, 5, 145, 242, 61,
                ]),
            ),
            (
                "h".to_string(),
                rmpv::Value::Binary(vec![
                    150, 183, 185, 103, 121, 185, 70, 194, 171, 206, 238, 163, 192, 250, 45, 88,
                ]),
            ),
            (
                "extra".to_string(),
                rmpv::Value::String("value".to_string().into()),
            ),
        ]))
        .unwrap();
        let encrypted = cipher.encrypt_vec(&encoded).unwrap();
        let input = BASE64URL_NOPAD.encode(&encrypted);

        let tags = RepoFileTags::from_string(&input, &cipher).unwrap();

        let expected_tags = RepoFileTags {
            encrypted_hash: Some(vec![
                10, 11, 64, 74, 230, 55, 68, 151, 148, 25, 24, 88, 5, 145, 242, 61,
            ]),
            hash: Some(vec![
                150, 183, 185, 103, 121, 185, 70, 194, 171, 206, 238, 163, 192, 250, 45, 88,
            ]),
            unknown: HashMap::from([("extra".into(), "value".into())]),
        };

        assert_eq!(tags, expected_tags)
    }

    #[test]
    fn from_string_base64_error() {
        let cipher = create_cipher();

        let err = RepoFileTags::from_string(&"a", &cipher).unwrap_err();

        assert!(matches!(err, RepoFileTagsDecodeError::Base64Error(..)));
        assert_eq!(
            err.user_error(),
            "Failed to base64 decode tags: invalid length at 0"
        );
    }

    #[test]
    fn from_string_decrypt_error() {
        let cipher = create_cipher();

        let err = RepoFileTags::from_string(&BASE64URL_NOPAD.encode("a".as_bytes()), &cipher)
            .unwrap_err();

        assert!(matches!(err, RepoFileTagsDecodeError::DecryptError(..)));
        assert_eq!(
            err.user_error(),
            "Failed to decrypt tags: file is too short to be decrypted"
        );
    }

    #[test]
    fn from_string_rmpserde_error() {
        let cipher = create_cipher();

        let err = RepoFileTags::from_string(
            &BASE64URL_NOPAD.encode(&cipher.encrypt_vec("a".as_bytes()).unwrap()),
            &cipher,
        )
        .unwrap_err();

        assert!(matches!(err, RepoFileTagsDecodeError::RMPSerdeError(..)));
        assert_eq!(
            err.user_error(),
            "Failed to deserialize tags: invalid type: integer `97`, expected struct RepoFileTags"
        );
    }

    #[test]
    fn to_string() {
        let cipher = create_cipher();

        let tags = RepoFileTags {
            encrypted_hash: Some(vec![
                10, 11, 64, 74, 230, 55, 68, 151, 148, 25, 24, 88, 5, 145, 242, 61,
            ]),
            hash: Some(vec![
                150, 183, 185, 103, 121, 185, 70, 194, 171, 206, 238, 163, 192, 250, 45, 88,
            ]),
            unknown: HashMap::from([("extra".into(), "value".into())]),
        };

        let encrypted = tags.to_string(&cipher).unwrap();

        let tags1 = RepoFileTags::from_string(&encrypted, &cipher).unwrap();

        assert_eq!(tags, tags1);
    }
}
