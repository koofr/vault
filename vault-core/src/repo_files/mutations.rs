use std::collections::HashSet;

use crate::{
    cipher::{data_cipher::decrypt_size, Cipher},
    file_types::{
        content_type::ext_to_content_type,
        file_category::{ext_to_file_category, FileCategory},
    },
    remote_files::{
        selectors as remote_files_selectors,
        state::{RemoteFile, RemoteFileType},
    },
    store,
    utils::{name_utils, path_utils},
};

use super::{
    errors::DecryptFilesError,
    selectors,
    state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize},
};

pub fn sort_children(state: &mut store::State, file_id: &str) {
    if let Some(children_ids) = state.repo_files.children.get(file_id) {
        state.repo_files.children.insert(
            file_id.to_owned(),
            selectors::select_sorted_files(state, &children_ids, &Default::default()),
        );
    }
}

pub fn decrypt_files(
    state: &mut store::State,
    repo_id: &str,
    path: &str,
    cipher: &Cipher,
) -> Result<(), DecryptFilesError> {
    let (mount_id, full_path) =
        selectors::select_repo_path_to_mount_path(state, repo_id, path, &cipher)?;

    let root_remote_file_id = remote_files_selectors::get_file_id(&mount_id, &full_path);

    if let Some(root_remote_file) = state.remote_files.files.get(&root_remote_file_id) {
        let root_repo_file = match path {
            "/" => get_root_file(repo_id, root_remote_file),
            _ => decrypt_file(
                repo_id,
                path_utils::parent_path(path).unwrap(),
                root_remote_file,
                &cipher,
            ),
        };
        let root_repo_file_id = root_repo_file.id.clone();

        state
            .repo_files
            .files
            .insert(root_repo_file_id.clone(), root_repo_file);

        if let Some(remote_children_ids) = state.remote_files.children.get(&root_remote_file_id) {
            let mut children = Vec::with_capacity(remote_children_ids.len());

            for remote_child in remote_children_ids
                .iter()
                .filter_map(|id| state.remote_files.files.get(id))
            {
                let repo_child = decrypt_file(repo_id, path, remote_child, &cipher);

                children.push(repo_child.id.clone());

                state
                    .repo_files
                    .files
                    .insert(repo_child.id.clone(), repo_child);
            }

            let children_set = children.clone().into_iter().collect::<HashSet<String>>();

            if let Some(old_children) = state.repo_files.children.get(&root_repo_file_id) {
                let old_children = old_children.clone();

                for old_child in old_children {
                    if !children_set.contains(&old_child) {
                        cleanup_file(state, &old_child);
                    }
                }
            }

            state
                .repo_files
                .children
                .insert(root_repo_file_id.clone(), children);

            sort_children(state, &root_repo_file_id);
        }

        if state
            .remote_files
            .loaded_roots
            .contains(&root_remote_file_id)
        {
            state
                .repo_files
                .loaded_roots
                .insert(root_repo_file_id.clone());
        }
    }

    Ok(())
}

pub fn decrypt_file(
    repo_id: &str,
    parent_path: &str,
    remote_file: &RemoteFile,
    cipher: &Cipher,
) -> RepoFile {
    let name = match cipher.decrypt_filename(&remote_file.name) {
        Ok(name) => {
            let name_lower = name.to_lowercase();

            RepoFileName::Decrypted { name, name_lower }
        }
        Err(err) => RepoFileName::DecryptError {
            encrypted_name: remote_file.name.clone(),
            encrypted_name_lower: remote_file.name_lower.clone(),
            error: err,
        },
    };
    let (path, id) = match &name {
        RepoFileName::Decrypted { name, .. } => {
            let path = path_utils::join_path_name(parent_path, &name);
            let id = selectors::get_file_id(repo_id, &path);

            (RepoFilePath::Decrypted { path }, id)
        }
        RepoFileName::DecryptError {
            encrypted_name,
            error,
            ..
        } => {
            let id = selectors::get_file_id(
                repo_id,
                &path_utils::join_path_name(parent_path, &encrypted_name),
            );

            (
                RepoFilePath::DecryptError {
                    parent_path: parent_path.to_owned(),
                    encrypted_name: encrypted_name.clone(),
                    error: error.clone(),
                },
                id,
            )
        }
    };
    let size = match remote_file.typ {
        RemoteFileType::File => match decrypt_size(remote_file.size) {
            Ok(size) => RepoFileSize::Decrypted { size },
            Err(err) => RepoFileSize::DecryptError {
                encrypted_size: remote_file.size,
                error: err,
            },
        },
        RemoteFileType::Dir => RepoFileSize::Decrypted { size: 0 },
    };
    let (ext, content_type, category) = match &remote_file.typ {
        RemoteFileType::File => match &name {
            RepoFileName::Decrypted { name_lower, .. } => {
                selectors::get_file_ext_content_type_category(name_lower)
            }
            RepoFileName::DecryptError { .. } => (None, None, FileCategory::Generic),
        },
        RemoteFileType::Dir => (None, None, FileCategory::Folder),
    };

    RepoFile {
        id,
        mount_id: remote_file.mount_id.clone(),
        remote_path: remote_file.path.clone(),
        repo_id: repo_id.to_owned(),
        path,
        name,
        ext,
        content_type,
        typ: (&remote_file.typ).into(),
        size,
        modified: remote_file.modified,
        category,
    }
}

pub fn get_root_file(repo_id: &str, remote_file: &RemoteFile) -> RepoFile {
    RepoFile {
        id: selectors::get_file_id(repo_id, "/"),
        mount_id: remote_file.mount_id.clone(),
        remote_path: remote_file.path.clone(),
        repo_id: repo_id.to_owned(),
        path: RepoFilePath::Decrypted {
            path: String::from("/"),
        },
        name: RepoFileName::Decrypted {
            name: String::from(""),
            name_lower: String::from(""),
        },
        ext: None,
        content_type: None,
        typ: super::state::RepoFileType::Dir,
        size: RepoFileSize::Decrypted { size: 0 },
        modified: 0,
        category: FileCategory::Folder,
    }
}

pub fn cleanup_file(state: &mut store::State, file_id: &str) {
    state.repo_files.files.remove(file_id);

    let file_id_prefix = if file_id.ends_with('/') {
        file_id.to_owned()
    } else {
        format!("{file_id}/")
    };

    state
        .repo_files
        .files
        .retain(|file_id, _| !file_id.starts_with(&file_id_prefix));

    state.repo_files.children.remove(file_id);

    state
        .repo_files
        .children
        .retain(|file_id, _| !file_id.starts_with(&file_id_prefix));
}

#[cfg(test)]
mod tests {
    use crate::{
        cipher::{
            errors::{DecryptFilenameError, DecryptSizeError},
            test_helpers::create_cipher,
        },
        file_types::file_category::FileCategory,
        remote_files::test_helpers as remote_files_test_helpers,
        repo_files::state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType},
    };

    use super::{decrypt_file, get_root_file};

    #[test]
    fn test_get_root_file() {
        let remote_file = remote_files_test_helpers::create_dir("m1", "/Vault");

        assert_eq!(
            get_root_file("r1", &remote_file),
            RepoFile {
                id: String::from("r1:/"),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: String::from("r1",),
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
                size: RepoFileSize::Decrypted { size: 0 },
                modified: 0,
                category: FileCategory::Folder,
            }
        )
    }

    #[test]
    fn test_decrypt_file_dir() {
        let cipher = create_cipher();
        let remote_file = remote_files_test_helpers::create_dir(
            "m1",
            &format!("/Vault/{}", cipher.encrypt_filename("D1")),
        );

        assert_eq!(
            decrypt_file("r1", "/", &remote_file, &cipher),
            RepoFile {
                id: String::from("r1:/D1"),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: String::from("r1",),
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
                size: RepoFileSize::Decrypted { size: 0 },
                modified: 1,
                category: FileCategory::Folder,
            }
        )
    }

    #[test]
    fn test_decrypt_file_dir_decrypt_error() {
        let cipher = create_cipher();
        let remote_file = remote_files_test_helpers::create_dir("m1", "/Vault/D1");

        assert_eq!(
            decrypt_file("r1", "/", &remote_file, &cipher),
            RepoFile {
                id: String::from("r1:/D1"),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: String::from("r1",),
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
                size: RepoFileSize::Decrypted { size: 0 },
                modified: 1,
                category: FileCategory::Folder,
            }
        )
    }

    #[test]
    fn test_decrypt_file_file() {
        let cipher = create_cipher();
        let remote_file = remote_files_test_helpers::create_file(
            "m1",
            &format!("/Vault/{}", cipher.encrypt_filename("Image.JPG")),
        );

        assert_eq!(
            decrypt_file("r1", "/", &remote_file, &cipher),
            RepoFile {
                id: String::from("r1:/Image.JPG"),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: String::from("r1",),
                path: RepoFilePath::Decrypted {
                    path: String::from("/Image.JPG")
                },
                name: RepoFileName::Decrypted {
                    name: String::from("Image.JPG"),
                    name_lower: String::from("image.jpg")
                },
                ext: Some(String::from("jpg")),
                content_type: Some(String::from("image/jpeg")),
                typ: RepoFileType::File,
                size: RepoFileSize::Decrypted { size: 52 },
                modified: 1,
                category: FileCategory::Image,
            }
        )
    }

    #[test]
    fn test_decrypt_file_file_decrypt_error() {
        let cipher = create_cipher();
        let mut remote_file = remote_files_test_helpers::create_file("m1", "/Vault/F1");
        remote_file.size = 10;

        assert_eq!(
            decrypt_file("r1", "/", &remote_file, &cipher),
            RepoFile {
                id: String::from("r1:/F1"),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: String::from("r1",),
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
                size: RepoFileSize::DecryptError {
                    encrypted_size: 10,
                    error: DecryptSizeError::EncryptedFileTooShort
                },
                modified: 1,
                category: FileCategory::Generic,
            }
        )
    }
}
