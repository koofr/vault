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
                    let parent_path =
                        path_utils::join_paths(root_path, &decrypted_item_parent_path);
                    let repo_file = repo_files_mutations::decrypt_file(
                        repo_id,
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
                    RemoteError::from_api_error_details(error, None),
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cipher::{errors::DecryptFilenameError, test_helpers::create_cipher},
        file_types::file_icon_type::FileIconType,
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
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/")),
                file: RepoFile {
                    id: String::from("r1:/"),
                    mount_id: String::from("m1"),
                    remote_path: String::from("/Vault"),
                    repo_id: String::from("r1",),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from(""),
                        name_lower: String::from("")
                    },
                    typ: RepoFileType::Dir,
                    size: RepoFileSize::Decrypted { size: 0 },
                    modified: 0,
                    icon_type: FileIconType::Folder,
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
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/D1")),
                file: RepoFile {
                    id: String::from("r1:/D1"),
                    mount_id: String::from("m1"),
                    remote_path: format!("/Vault/{}", cipher.encrypt_filename("D1")),
                    repo_id: String::from("r1"),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/D1")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from("D1"),
                        name_lower: String::from("d1")
                    },
                    typ: RepoFileType::Dir,
                    size: RepoFileSize::Decrypted { size: 0 },
                    modified: 1,
                    icon_type: FileIconType::Folder,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_dir_decrypt_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_dir("/D1", "D1");

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecodeError(String::from(
                    "non-zero trailing bits at 1"
                ))),
                file: RepoFile {
                    id: String::from("r1:/D1"),
                    mount_id: String::from("m1"),
                    remote_path: String::from("/Vault/D1"),
                    repo_id: String::from("r1"),
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
                    typ: RepoFileType::Dir,
                    size: RepoFileSize::Decrypted { size: 0 },
                    modified: 1,
                    icon_type: FileIconType::Folder,
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
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Ok(String::from("/F1")),
                file: RepoFile {
                    id: String::from("r1:/F1"),
                    mount_id: String::from("m1"),
                    remote_path: format!("/Vault/{}", cipher.encrypt_filename("F1")),
                    repo_id: String::from("r1"),
                    path: RepoFilePath::Decrypted {
                        path: String::from("/F1")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from("F1"),
                        name_lower: String::from("f1")
                    },
                    typ: RepoFileType::File,
                    size: RepoFileSize::Decrypted { size: 52 },
                    modified: 1,
                    icon_type: FileIconType::Generic,
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
                    path: RepoFilePath::Decrypted {
                        path: String::from("/D1/F1")
                    },
                    name: RepoFileName::Decrypted {
                        name: String::from("F1"),
                        name_lower: String::from("f1")
                    },
                    typ: RepoFileType::File,
                    size: RepoFileSize::Decrypted { size: 52 },
                    modified: 1,
                    icon_type: FileIconType::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_decrypt_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file("/F1", "F1");

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", item, &cipher),
            RepoFilesListRecursiveItem::File {
                relative_repo_path: Err(DecryptFilenameError::DecodeError(String::from(
                    "non-zero trailing bits at 1"
                ))),
                file: RepoFile {
                    id: String::from("r1:/F1"),
                    mount_id: String::from("m1"),
                    remote_path: String::from("/Vault/F1"),
                    repo_id: String::from("r1"),
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
                    typ: RepoFileType::File,
                    size: RepoFileSize::Decrypted { size: 52 },
                    modified: 1,
                    icon_type: FileIconType::Generic,
                },
            }
        )
    }

    #[test]
    fn test_decrypt_files_list_recursive_item_file_decrypt_parent_path_error() {
        let cipher = create_cipher();
        let item = remote_test_helpers::create_files_list_recursive_item_file("/D1/F1", "F1");

        assert_eq!(
            decrypt_files_list_recursive_item("m1", "/Vault", "r1", "/", item, &cipher),
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
