use crate::{
    cipher::Cipher,
    remote::{models, RemoteError},
    remote_files::{mutations as remote_files_mutations, selectors as remote_files_selectors},
    repo_files::mutations as repo_files_mutations,
    utils::path_utils,
};

use super::{errors::FilesListRecursiveItemError, state::RepoFilesListRecursiveItem};

pub fn decrypt_files_list_recursive_item(
    mount_id: &str,
    root_remote_path: &str,
    repo_id: &str,
    encrypted_root_path: &str,
    root_path: &str,
    item: models::FilesListRecursiveItem,
    cipher: &Cipher,
) -> RepoFilesListRecursiveItem {
    match item {
        models::FilesListRecursiveItem::File {
            path: remote_item_path,
            file,
        } => {
            let remote_path = path_utils::join_paths(&root_remote_path, &remote_item_path);
            let remote_file_id = remote_files_selectors::get_file_id(mount_id, &remote_path);
            let remote_file = remote_files_mutations::files_file_to_remote_file(
                remote_file_id,
                mount_id.to_owned(),
                remote_path.clone(),
                file,
            );
            let (repo_file, relative_repo_path) = match remote_item_path.as_str() {
                "/" => (
                    repo_files_mutations::get_root_file(repo_id, &remote_file),
                    Ok(String::from("/")),
                ),
                _ => {
                    let encrypted_item_parent_path =
                        path_utils::parent_path(&remote_item_path).unwrap();
                    let decrypted_item_parent_path =
                        match cipher.decrypt_path(&encrypted_item_parent_path) {
                            Ok(path) => path,
                            Err(err) => {
                                return RepoFilesListRecursiveItem::Error {
                                    mount_id: mount_id.to_owned(),
                                    remote_path: Some(remote_path),
                                    error: FilesListRecursiveItemError::DecryptFilenameError(err),
                                }
                            }
                        };
                    let encrypted_parent_path =
                        path_utils::join_paths(encrypted_root_path, encrypted_item_parent_path);
                    let parent_path =
                        path_utils::join_paths(root_path, &decrypted_item_parent_path);
                    let repo_file = repo_files_mutations::decrypt_file(
                        repo_id,
                        &encrypted_parent_path,
                        &parent_path,
                        &remote_file,
                        &cipher,
                    );
                    let relative_repo_path = repo_file
                        .decrypted_name()
                        .map(|name| path_utils::join_path_name(&decrypted_item_parent_path, name));

                    (repo_file, relative_repo_path)
                }
            };
            RepoFilesListRecursiveItem::File {
                relative_repo_path,
                file: repo_file,
            }
        }
        models::FilesListRecursiveItem::Error { path, error } => {
            RepoFilesListRecursiveItem::Error {
                mount_id: mount_id.to_owned(),
                remote_path: path.map(|path| path_utils::join_paths(&root_remote_path, &path)),
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
        repo_files_list::{errors::FilesListRecursiveItemError, state::RepoFilesListRecursiveItem},
    };

    use super::decrypt_files_list_recursive_item;

    #[test]
    fn test_decrypt_files_list_recursive_item_root() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_dir("/", "");

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/")),
                file: RepoFile {
                    id: String::from("r1:/"),
                    mount_id: String::from("m1"),
                    remote_path: String::from("/Vault"),
                    repo_id: String::from("r1"),
                    encrypted_path: String::from("/"),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from(""),
                        name_lower: String::from("")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::Dir,
                    size: None,
                    modified: None,
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
            &format!("/{}", cipher.encrypt_filename("D1")),
            &cipher.encrypt_filename("D1"),
        );

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/D1")),
                file: RepoFile {
                    id: String::from("r1:/D1"),
                    mount_id: String::from("m1"),
                    remote_path: format!("/Vault/{}", cipher.encrypt_filename("D1")),
                    repo_id: String::from("r1"),
                    encrypted_path: format!("/{}", cipher.encrypt_filename("D1")),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/D1")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from("D1"),
                        name_lower: String::from("d1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::Dir,
                    size: None,
                    modified: None,
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
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecodeError(String::from(
                    "non-zero trailing bits at 1"
                ))),
                file: RepoFile {
                    id: String::from("err:r1:/D1"),
                    mount_id: String::from("m1"),
                    remote_path: String::from("/Vault/D1"),
                    repo_id: String::from("r1"),
                    encrypted_path: String::from("/D1"),
                    path: RepoFilePath::DecryptError {
                        parent_path: String::from("/"),
                        encrypted_name: String::from("D1"),
                        error: DecryptFilenameError::DecodeError(String::from(
                            "non-zero trailing bits at 1"
                        )),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: String::from("D1"),
                        encrypted_name_lower: String::from("d1"),
                        error: DecryptFilenameError::DecodeError(String::from(
                            "non-zero trailing bits at 1"
                        )),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::Dir,
                    size: None,
                    modified: None,
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
            &format!("/{}", cipher.encrypt_filename("F1")),
            &cipher.encrypt_filename("F1"),
        );

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/F1")),
                file: RepoFile {
                    id: String::from("r1:/F1"),
                    mount_id: String::from("m1"),
                    remote_path: format!("/Vault/{}", cipher.encrypt_filename("F1")),
                    repo_id: String::from("r1"),
                    encrypted_path: format!("/{}", cipher.encrypt_filename("F1")),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/F1")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from("F1"),
                        name_lower: String::from("f1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
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
            &format!("/{}", cipher.encrypt_filename("F1")),
            &cipher.encrypt_filename("F1"),
        );

        assert_eq!(
            decrypt_files_list_recursive_item(
                "m1",
                &format!("/Vault/{}", cipher.encrypt_filename("D1")),
                "r1",
                &format!("/{}", cipher.encrypt_filename("D1")),
                "/D1",
                item,
                &cipher
            ),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/F1")),
                file: RepoFile {
                    id: String::from("r1:/D1/F1"),
                    mount_id: String::from("m1"),
                    remote_path: format!(
                        "/Vault/{}/{}",
                        cipher.encrypt_filename("D1"),
                        cipher.encrypt_filename("F1")
                    ),
                    repo_id: String::from("r1"),
                    encrypted_path: format!(
                        "/{}/{}",
                        cipher.encrypt_filename("D1"),
                        cipher.encrypt_filename("F1")
                    ),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/D1/F1")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from("F1"),
                        name_lower: String::from("f1")
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
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
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecodeError(String::from(
                    "non-zero trailing bits at 1"
                ))),
                file: RepoFile {
                    id: String::from("err:r1:/F1"),
                    mount_id: String::from("m1"),
                    remote_path: String::from("/Vault/F1"),
                    repo_id: String::from("r1"),
                    encrypted_path: String::from("/F1"),
                    path: RepoFilePath::DecryptError {
                        parent_path: String::from("/"),
                        encrypted_name: String::from("F1"),
                        error: DecryptFilenameError::DecodeError(String::from(
                            "non-zero trailing bits at 1"
                        )),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: String::from("F1"),
                        encrypted_name_lower: String::from("f1"),
                        error: DecryptFilenameError::DecodeError(String::from(
                            "non-zero trailing bits at 1"
                        )),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(RepoFileSize::Decrypted { size: 52 }),
                    modified: Some(1),
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
        let item = remote_test_helpers::create_files_list_recursive_item_file("/D1/F1", "F1");

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", "/", item, &cipher),
            RepoFilesListRecursiveItem::Error {
                mount_id: String::from("m1"),
                remote_path: Some(String::from("/Vault/D1/F1")),
                error: FilesListRecursiveItemError::DecryptFilenameError(
                    DecryptFilenameError::DecodeError(String::from("non-zero trailing bits at 1"))
                ),
            }
        )
    }
}
