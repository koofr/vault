use crate::{
    cipher::Cipher,
    remote_files::state::{RemoteFile, RemoteFileType},
};

use super::{errors::DecryptTagsError, selectors::REMOTE_FILE_TAGS_KEY, state::RepoFileTags};

pub fn decrypt_tags(
    remote_file: &RemoteFile,
    cipher: &Cipher,
) -> Option<Result<RepoFileTags, DecryptTagsError>> {
    let values = remote_file.tags.get(REMOTE_FILE_TAGS_KEY)?;

    if values.len() > 1 {
        return Some(Err(DecryptTagsError::MultipleValues));
    }

    let value = values.first()?;

    let tags = match RepoFileTags::from_string(value, &cipher) {
        Ok(tags) => tags,
        Err(err) => return Some(Err(DecryptTagsError::DecodeError(err))),
    };

    let tags_encrypted_hash_hex = tags.encrypted_hash_hex();

    match (&remote_file.typ, &remote_file.hash, tags_encrypted_hash_hex) {
        (RemoteFileType::Dir, _, _) => {}
        (RemoteFileType::File, Some(remote_file_hash), Some(tags_encrypted_hash_hex)) => {
            if &tags_encrypted_hash_hex != remote_file_hash {
                return Some(Err(DecryptTagsError::EncryptedHashMismatch {
                    expected_encrypted_hash: Some(tags_encrypted_hash_hex),
                    encrypted_hash: Some(remote_file_hash.to_owned()),
                }));
            }
        }
        (RemoteFileType::File, _, tags_encrypted_hash_hex) => {
            return Some(Err(DecryptTagsError::EncryptedHashMismatch {
                expected_encrypted_hash: tags_encrypted_hash_hex,
                encrypted_hash: None,
            }))
        }
    }

    Some(Ok(tags))
}
