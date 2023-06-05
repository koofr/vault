use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    cipher::{data_cipher::decrypt_size, Cipher},
    file_types::file_category::FileCategory,
    remote_files::{
        selectors as remote_files_selectors,
        state::{RemoteFile, RemoteFileType},
    },
    store,
    utils::path_utils,
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

fn remote_files_to_repo_files<'a>(
    state: &'a store::State,
    remote_files: impl Iterator<Item = (&'a str, &'a str)> + 'a,
) -> impl Iterator<Item = (String, String, String, String)> + 'a {
    remote_files.flat_map(|(mount_id, remote_path)| {
        if let Some(repo_tree) = state.repos.mount_repo_trees.get(mount_id) {
            repo_tree
                .get(&remote_path)
                .into_iter()
                .map(|(repo_id, path)| {
                    (
                        mount_id.to_owned(),
                        remote_path.to_owned(),
                        repo_id.to_owned(),
                        path.to_owned(),
                    )
                })
                .collect()
        } else {
            vec![]
        }
    })
}

pub fn handle_remote_files_mutation(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    ciphers: &HashMap<String, Arc<Cipher>>,
) {
    let remote_loaded_roots = mutation_state
        .remote_files
        .loaded_roots
        .iter()
        .map(|(mount_id, path)| (mount_id.as_str(), path.as_str()));
    let remote_created_files = mutation_state
        .remote_files
        .created_files
        .iter()
        .map(|(mount_id, path)| (mount_id.as_str(), path.as_str()));
    let remote_created_files_parents =
        mutation_state
            .remote_files
            .created_files
            .iter()
            .filter_map(|(mount_id, path)| {
                path_utils::parent_path(path).map(|parent_path| (mount_id.as_str(), parent_path))
            });
    let remote_removed_files = mutation_state
        .remote_files
        .removed_files
        .iter()
        .map(|(mount_id, path)| (mount_id.as_str(), path.as_str()));
    let remote_removed_files_parents =
        mutation_state
            .remote_files
            .removed_files
            .iter()
            .filter_map(|(mount_id, path)| {
                path_utils::parent_path(path).map(|parent_path| (mount_id.as_str(), parent_path))
            });
    let remote_moved_from_files = mutation_state
        .remote_files
        .moved_files
        .iter()
        .map(|(mount_id, old_path, _)| (mount_id.as_str(), old_path.as_str()));
    let remote_moved_from_files_parents = mutation_state
        .remote_files
        .moved_files
        .iter()
        .filter_map(|(mount_id, old_path, _)| {
            path_utils::parent_path(old_path)
                .map(|old_parent_path| (mount_id.as_str(), old_parent_path))
        });
    let remote_moved_to_files = mutation_state
        .remote_files
        .moved_files
        .iter()
        .map(|(mount_id, _, new_path)| (mount_id.as_str(), new_path.as_str()));
    let remote_moved_to_files_parents =
        mutation_state
            .remote_files
            .moved_files
            .iter()
            .filter_map(|(mount_id, _, new_path)| {
                path_utils::parent_path(new_path)
                    .map(|new_parent_path| (mount_id.as_str(), new_parent_path))
            });

    let files_to_decrypt: HashSet<(String, String, String, String)> = remote_files_to_repo_files(
        state,
        remote_loaded_roots
            .chain(remote_created_files)
            .chain(remote_created_files_parents)
            .chain(remote_removed_files)
            .chain(remote_removed_files_parents)
            .chain(remote_moved_from_files)
            .chain(remote_moved_from_files_parents)
            .chain(remote_moved_to_files)
            .chain(remote_moved_to_files_parents),
    )
    .collect();

    let mut repo_files_dirty = false;

    for (mount_id, remote_path, repo_id, path) in files_to_decrypt {
        if let Some(cipher) = ciphers.get(&repo_id) {
            let _ = decrypt_files(
                state,
                &mount_id,
                &remote_path,
                &repo_id,
                &path,
                cipher.as_ref(),
            );

            repo_files_dirty = true;
        }
    }

    if repo_files_dirty {
        notify(store::Event::RepoFiles);
    }

    let removed_repo_files = remote_files_to_repo_files(
        state,
        mutation_state
            .remote_files
            .removed_files
            .iter()
            .map(|(mount_id, path)| (mount_id.as_str(), path.as_str())),
    )
    .map(|(_, _, repo_id, path)| (repo_id, path));
    let moved_repo_files_from = remote_files_to_repo_files(
        state,
        mutation_state
            .remote_files
            .moved_files
            .iter()
            .map(|(mount_id, old_path, _)| (mount_id.as_str(), old_path.as_str())),
    )
    .map(|(_, _, repo_id, path)| (repo_id, path));
    let moved_repo_files_to = remote_files_to_repo_files(
        state,
        mutation_state
            .remote_files
            .moved_files
            .iter()
            .map(|(mount_id, _, new_path)| (mount_id.as_str(), new_path.as_str())),
    )
    .map(|(_, _, repo_id, path)| (repo_id, path));
    let moved_repo_files: Vec<(String, String, String)> = moved_repo_files_from
        .zip(moved_repo_files_to)
        .filter_map(
            |((from_repo_id, from_repo_path), (to_repo_id, to_repo_path))| {
                if from_repo_id == to_repo_id {
                    Some((from_repo_id, from_repo_path, to_repo_path))
                } else {
                    None
                }
            },
        )
        .collect();

    for (repo_id, path) in removed_repo_files {
        if let Some(cipher) = ciphers.get(&repo_id) {
            if let Ok(path) = cipher.decrypt_path(&path) {
                mutation_state
                    .repo_files
                    .removed_files
                    .push((repo_id, path));
            }
        }
    }

    for (repo_id, from_path, to_path) in moved_repo_files {
        if let Some(cipher) = ciphers.get(&repo_id) {
            if let Ok(from_path) = cipher.decrypt_path(&from_path) {
                if let Ok(to_path) = cipher.decrypt_path(&to_path) {
                    mutation_state
                        .repo_files
                        .moved_files
                        .push((repo_id, from_path, to_path));
                }
            }
        }
    }

    if repo_files_dirty {
        mutation_notify(store::MutationEvent::RepoFiles, state, mutation_state);
    }
}

pub fn decrypt_files(
    state: &mut store::State,
    mount_id: &str,
    remote_path: &str,
    repo_id: &str,
    encrypted_path: &str,
    cipher: &Cipher,
) -> Result<(), DecryptFilesError> {
    let root_remote_file_id = remote_files_selectors::get_file_id(mount_id, remote_path);

    if let Some(root_remote_file) = state.remote_files.files.get(&root_remote_file_id) {
        let root_repo_file = match encrypted_path {
            "/" => get_root_file(repo_id, root_remote_file),
            _ => decrypt_file(
                repo_id,
                &cipher.decrypt_path(path_utils::parent_path(&encrypted_path).unwrap())?,
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
            let path = cipher.decrypt_path(encrypted_path)?;

            let mut children = Vec::with_capacity(remote_children_ids.len());

            for remote_child in remote_children_ids
                .iter()
                .filter_map(|id| state.remote_files.files.get(id))
            {
                let repo_child = decrypt_file(repo_id, &path, remote_child, &cipher);

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
    } else {
        if let Ok(path) = cipher.decrypt_path(encrypted_path) {
            let file_id = selectors::get_file_id(repo_id, &path);

            state.repo_files.files.remove(&file_id);
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
    let unique_name = selectors::get_file_unique_name(&remote_file.unique_id, ext.as_deref());

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
        unique_name,
        remote_hash: remote_file.hash.clone(),
        category,
    }
}

pub fn get_root_file(repo_id: &str, remote_file: &RemoteFile) -> RepoFile {
    let unique_name = selectors::get_file_unique_name(&remote_file.unique_id, None);

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
        unique_name,
        remote_hash: None,
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
    use similar_asserts::assert_eq;

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
                unique_name: String::from("2b6bea08149b89711b061f1291492d46"),
                remote_hash: None,
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
                modified: 0,
                unique_name: String::from("4d6bb967e30d7a5d36c3e6b607d71cf2"),
                remote_hash: None,
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
                modified: 0,
                unique_name: String::from("a2216f6522ef8e23512f13d37592b43b"),
                remote_hash: None,
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
                unique_name: String::from("c7f010983b2f25f3e1d604c2870d82c8.jpg"),
                remote_hash: Some(String::from("hash")),
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
                unique_name: String::from("de40e3afb025fe16012fd421e246c711"),
                remote_hash: Some(String::from("hash")),
                category: FileCategory::Generic,
            }
        )
    }
}
