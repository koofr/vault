use crate::{
    cipher::{errors::DecryptFilenameError, Cipher},
    remote::{models, RemoteError},
    remote_files::{mutations as remote_files_mutations, selectors as remote_files_selectors},
    repo_files::mutations as repo_files_mutations,
    types::{DecryptedPath, EncryptedPath, MountId, RemotePath, RepoId},
    utils::{path_utils, remote_path_utils, repo_path_utils},
};

use super::{errors::FilesListRecursiveItemError, state::RepoFilesListRecursiveItem};

pub fn decrypt_files_list_recursive_item(
    mount_id: &MountId,
    root_remote_path: &RemotePath,
    repo_id: &RepoId,
    encrypted_root_path: &EncryptedPath,
    root_path: &Result<DecryptedPath, DecryptFilenameError>,
    item: models::FilesListRecursiveItem,
    cipher: &Cipher,
) -> RepoFilesListRecursiveItem {
    match item {
        models::FilesListRecursiveItem::File {
            path: remote_item_path,
            file,
        } => {
            let remote_path = remote_path_utils::join_paths(&root_remote_path, &remote_item_path);
            let remote_file_id =
                remote_files_selectors::get_file_id(mount_id, &remote_path.to_lowercase());
            let remote_file = remote_files_mutations::files_file_to_remote_file(
                remote_file_id,
                mount_id.to_owned(),
                remote_path.clone(),
                file,
            );
            let (repo_file, relative_repo_path) = if remote_item_path.is_root() {
                (
                    repo_files_mutations::get_root_file(repo_id, &remote_file),
                    Ok(DecryptedPath("/".into())),
                )
            } else {
                let encrypted_item_parent_path = EncryptedPath(
                    path_utils::parent_path(&remote_item_path.0)
                        .unwrap()
                        .to_owned(),
                );
                let decrypted_item_parent_path = cipher.decrypt_path(&encrypted_item_parent_path);
                let encrypted_parent_path = EncryptedPath(path_utils::join_paths(
                    &encrypted_root_path.0,
                    &encrypted_item_parent_path.0,
                ));
                let parent_path = match (root_path, &decrypted_item_parent_path) {
                    (Ok(root_path), Ok(decrypted_item_parent_path)) => Ok(
                        repo_path_utils::join_paths(root_path, decrypted_item_parent_path),
                    ),
                    (Err(err), _) => Err(err.to_owned()),
                    (_, Err(err)) => Err(err.to_owned()),
                };
                let repo_file = repo_files_mutations::decrypt_file(
                    repo_id,
                    &encrypted_parent_path,
                    &parent_path,
                    &remote_file,
                    &cipher,
                );
                let relative_repo_path =
                    match (&decrypted_item_parent_path, repo_file.decrypted_name()) {
                        (Ok(decrypted_item_parent_path), Ok(decrypted_name)) => {
                            Ok(repo_path_utils::join_path_name(
                                decrypted_item_parent_path,
                                decrypted_name,
                            ))
                        }
                        (Err(err), _) => Err(err.to_owned()),
                        (_, Err(err)) => Err(err.to_owned()),
                    };

                (repo_file, relative_repo_path)
            };
            RepoFilesListRecursiveItem::File {
                relative_repo_path,
                file: repo_file,
            }
        }
        models::FilesListRecursiveItem::Error { path, error } => {
            RepoFilesListRecursiveItem::Error {
                mount_id: mount_id.to_owned(),
                remote_path: path
                    .map(|path| remote_path_utils::join_paths(&root_remote_path, &path)),
                error: FilesListRecursiveItemError::RemoteError(
                    RemoteError::from_api_error_details(error, None, None),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::{
        cipher::{errors::DecryptFilenameError, test_helpers::create_cipher},
        files::file_category::FileCategory,
        remote::test_helpers as remote_test_helpers,
        repo_files::state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType},
        repo_files_list::state::RepoFilesListRecursiveItem,
        types::{
            DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, MountId, RemotePath,
            RepoFileId, RepoId,
        },
    };

    use super::decrypt_files_list_recursive_item;

    #[test]
    fn test_decrypt_files_list_recursive_item_root() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_dir("/", "");

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(DecryptedPath("/".into())),
                file: RepoFile {
                    id: RepoFileId("r1:/".into()),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath("/Vault".into()),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath("/".into()),
                    path: RepoFilePath::Decrypted {
                        path: DecryptedPath("/".into())
                    },
                    name: RepoFileName::Decrypted {
                        name: DecryptedName("".into()),
                        name_lower: String::from("")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::Dir,
                    size: None,
                    modified: None,
                    tags: None,
                    unique_name: String::from("2b6bea08149b89711b061f1291492d46"),
                    remote_hash: None,
                    category: FileCategory::Folder,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_dir() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_dir(
            &format!(
                "/{}",
                cipher.encrypt_filename(&DecryptedName("D1".into())).0
            ),
            &cipher.encrypt_filename(&DecryptedName("D1".into())).0,
        );

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(DecryptedPath("/D1".into())),
                file: RepoFile {
                    id: RepoFileId(format!(
                        "r1:/{}",
                        cipher.encrypt_filename(&DecryptedName("D1".into())).0
                    )),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath(format!(
                        "/Vault/{}",
                        cipher.encrypt_filename(&DecryptedName("D1".into())).0
                    )),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath(format!(
                        "/{}",
                        cipher.encrypt_filename(&DecryptedName("D1".into())).0
                    )),
                    path: RepoFilePath::Decrypted {
                        path: DecryptedPath("/D1".into())
                    },
                    name: RepoFileName::Decrypted {
                        name: DecryptedName("D1".into()),
                        name_lower: String::from("d1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::Dir,
                    size: None,
                    modified: None,
                    tags: None,
                    unique_name: String::from("4d6bb967e30d7a5d36c3e6b607d71cf2"),
                    remote_hash: None,
                    category: FileCategory::Folder,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_dir_decrypt_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_dir("/D1", "D1");

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
                file: RepoFile {
                    id: RepoFileId("r1:/D1".into()),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath("/Vault/D1".into()),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath("/D1".into()),
                    path: RepoFilePath::DecryptError {
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("D1".into()),
                        encrypted_name_lower: String::from("d1"),
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::Dir,
                    size: None,
                    modified: None,
                    tags: None,
                    unique_name: String::from("a2216f6522ef8e23512f13d37592b43b"),
                    remote_hash: None,
                    category: FileCategory::Folder,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file(
            &format!(
                "/{}",
                cipher.encrypt_filename(&DecryptedName("F1".into())).0
            ),
            &cipher.encrypt_filename(&DecryptedName("F1".into())).0,
        );

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(DecryptedPath("/F1".into())),
                file: RepoFile {
                    id: RepoFileId(format!(
                        "r1:/{}",
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath(format!(
                        "/Vault/{}",
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath(format!(
                        "/{}",
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    path: RepoFilePath::Decrypted {
                        path: DecryptedPath("/F1".into())
                    },
                    name: RepoFileName::Decrypted {
                        name: DecryptedName("F1".into()),
                        name_lower: String::from("f1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
                    tags: None,
                    unique_name: String::from("f18ef102d1b034140d55e9dd8627a85b"),
                    remote_hash: Some(String::from("hash")),
                    category: FileCategory::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_non_root() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file(
            &format!(
                "/{}",
                cipher.encrypt_filename(&DecryptedName("F1".into())).0
            ),
            &cipher.encrypt_filename(&DecryptedName("F1".into())).0,
        );

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath(format!(
                    "/Vault/{}",
                    cipher.encrypt_filename(&DecryptedName("D1".into())).0
                )),
                &RepoId("r1".into()),
                &EncryptedPath(format!(
                    "/{}",
                    cipher.encrypt_filename(&DecryptedName("D1".into())).0
                )),
                &Ok(DecryptedPath("/D1".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(DecryptedPath("/F1".into())),
                file: RepoFile {
                    id: RepoFileId(format!(
                        "r1:/{}/{}",
                        cipher.encrypt_filename(&DecryptedName("D1".into())).0,
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath(format!(
                        "/Vault/{}/{}",
                        cipher.encrypt_filename(&DecryptedName("D1".into())).0,
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath(format!(
                        "/{}/{}",
                        cipher.encrypt_filename(&DecryptedName("D1".into())).0,
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    path: RepoFilePath::Decrypted {
                        path: DecryptedPath("/D1/F1".into())
                    },
                    name: RepoFileName::Decrypted {
                        name: DecryptedName("F1".into()),
                        name_lower: String::from("f1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
                    tags: None,
                    unique_name: String::from("ccda3acba4b41d4b94c808594a9cc689"),
                    remote_hash: Some(String::from("hash")),
                    category: FileCategory::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_decrypt_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file("/F1", "F1");

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
                file: RepoFile {
                    id: RepoFileId("r1:/F1".into()),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath("/Vault/F1".into()),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath("/F1".into()),
                    path: RepoFilePath::DecryptError {
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("F1".into()),
                        encrypted_name_lower: String::from("f1"),
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
                    tags: None,
                    unique_name: String::from("de40e3afb025fe16012fd421e246c711"),
                    remote_hash: Some(String::from("hash")),
                    category: FileCategory::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_decrypt_parent_path_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file(
            &format!(
                "/D1/{}",
                cipher.encrypt_filename(&DecryptedName("F1".into())).0
            ),
            &cipher.encrypt_filename(&DecryptedName("F1".into())).0,
        );

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
                file: RepoFile {
                    id: RepoFileId(format!(
                        "r1:/D1/{}",
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath(format!(
                        "/Vault/D1/{}",
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath(format!(
                        "/D1/{}",
                        cipher.encrypt_filename(&DecryptedName("F1".into())).0
                    )),
                    path: RepoFilePath::DecryptError {
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    name: RepoFileName::Decrypted {
                        name: DecryptedName("F1".into()),
                        name_lower: String::from("f1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
                    tags: None,
                    unique_name: String::from("c50276d197b1b9ea9b92d674e1d9c291"),
                    remote_hash: Some(String::from("hash")),
                    category: FileCategory::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_decrypt_parent_path_file_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file("/D1/F1", "F1");

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
                file: RepoFile {
                    id: RepoFileId("r1:/D1/F1".into()),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath("/Vault/D1/F1".into()),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath("/D1/F1".into()),
                    path: RepoFilePath::DecryptError {
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("F1".into()),
                        encrypted_name_lower: String::from("f1"),
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
                    tags: None,
                    unique_name: String::from("e73e9572f66d72dfbca3ad23eaebd8b5"),
                    remote_hash: Some(String::from("hash")),
                    category: FileCategory::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_decrypt_root_parent_path_file_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file("/D1/F1", "F1");

        assert_eq!(
            decrypt_files_list_recursive_item(
                &MountId("m1".into()),
                &RemotePath("/Vault".into()),
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
                file: RepoFile {
                    id: RepoFileId("r1:/D1/F1".into()),
                    mount_id: MountId("m1".into()),
                    remote_path: RemotePath("/Vault/D1/F1".into()),
                    repo_id: RepoId("r1".into()),
                    encrypted_path: EncryptedPath("/D1/F1".into()),
                    path: RepoFilePath::DecryptError {
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("F1".into()),
                        encrypted_name_lower: String::from("f1"),
                        error: DecryptFilenameError::DecryptFilenameError(
                            vault_crypto::errors::DecryptFilenameError::DecodeError(
                                "non-zero trailing bits at 1".into()
                            )
                        ),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
                    tags: None,
                    unique_name: String::from("e73e9572f66d72dfbca3ad23eaebd8b5"),
                    remote_hash: Some(String::from("hash")),
                    category: FileCategory::Generic,
                },
            }
        )
    }
}
