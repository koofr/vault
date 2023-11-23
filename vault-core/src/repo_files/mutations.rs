use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    cipher::{data_cipher::decrypt_size, errors::DecryptFilenameError, Cipher},
    files::file_category::FileCategory,
    remote_files::{
        selectors as remote_files_selectors,
        state::{RemoteFile, RemoteFileType},
    },
    store,
    types::{
        DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, MountId, RemotePath,
        RemotePathLower, RepoFileId, RepoId, ENCRYPTED_PATH_ROOT,
    },
    utils::{name_utils, path_utils, remote_path_utils, repo_path_utils},
};

use super::{
    errors::DecryptFilesError,
    selectors,
    state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize},
};

pub fn sort_children(state: &mut store::State, file_id: RepoFileId) {
    if let Some(children_ids) = state.repo_files.children.get(&file_id) {
        state.repo_files.children.insert(
            file_id,
            selectors::select_sorted_files(state, &children_ids, &Default::default()),
        );
    }
}

fn remote_files_to_repo_files<'a>(
    state: &'a store::State,
    remote_files: impl Iterator<Item = (&'a MountId, RemotePath)> + 'a,
) -> impl Iterator<Item = (MountId, RemotePath, RepoId, EncryptedPath)> + 'a {
    remote_files.flat_map(|(mount_id, remote_path)| {
        if let Some(repo_tree) = state.repos.mount_repo_trees.get(mount_id) {
            repo_tree
                .get(&remote_path)
                .into_iter()
                .map(|(repo_id, encrypted_path)| {
                    (
                        mount_id.to_owned(),
                        remote_path.to_owned(),
                        repo_id.to_owned(),
                        encrypted_path,
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
    ciphers: &HashMap<RepoId, Arc<Cipher>>,
) {
    let remote_loaded_roots = mutation_state
        .remote_files
        .loaded_roots
        .iter()
        .map(|(mount_id, path)| (mount_id, path.to_owned()));
    let remote_created_files = mutation_state
        .remote_files
        .created_files
        .iter()
        .map(|(mount_id, path)| (mount_id, path.to_owned()));
    let remote_created_files_parents =
        mutation_state
            .remote_files
            .created_files
            .iter()
            .filter_map(|(mount_id, path)| {
                remote_path_utils::parent_path(path).map(|parent_path| (mount_id, parent_path))
            });
    let remote_removed_files = mutation_state
        .remote_files
        .removed_files
        .iter()
        .map(|(mount_id, path)| (mount_id, path.to_owned()));
    let remote_removed_files_parents =
        mutation_state
            .remote_files
            .removed_files
            .iter()
            .filter_map(|(mount_id, path)| {
                remote_path_utils::parent_path(path).map(|parent_path| (mount_id, parent_path))
            });
    let remote_moved_from_files = mutation_state
        .remote_files
        .moved_files
        .iter()
        .map(|(mount_id, old_path, _)| (mount_id, old_path.to_owned()));
    let remote_moved_from_files_parents = mutation_state
        .remote_files
        .moved_files
        .iter()
        .filter_map(|(mount_id, old_path, _)| {
            remote_path_utils::parent_path(old_path)
                .map(|old_parent_path| (mount_id, old_parent_path))
        });
    let remote_moved_to_files = mutation_state
        .remote_files
        .moved_files
        .iter()
        .map(|(mount_id, _, new_path)| (mount_id, new_path.to_owned()));
    let remote_moved_to_files_parents =
        mutation_state
            .remote_files
            .moved_files
            .iter()
            .filter_map(|(mount_id, _, new_path)| {
                remote_path_utils::parent_path(new_path)
                    .map(|new_parent_path| (mount_id, new_parent_path))
            });

    let files_to_decrypt: HashSet<(MountId, RemotePath, RepoId, EncryptedPath)> =
        remote_files_to_repo_files(
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
            .map(|(mount_id, path)| (mount_id, path.to_owned())),
    )
    .map(|(_, _, repo_id, path)| (repo_id, path));
    let moved_repo_files_from = remote_files_to_repo_files(
        state,
        mutation_state
            .remote_files
            .moved_files
            .iter()
            .map(|(mount_id, old_path, _)| (mount_id, old_path.to_owned())),
    )
    .map(|(_, _, repo_id, path)| (repo_id, path));
    let moved_repo_files_to = remote_files_to_repo_files(
        state,
        mutation_state
            .remote_files
            .moved_files
            .iter()
            .map(|(mount_id, _, new_path)| (mount_id, new_path.to_owned())),
    )
    .map(|(_, _, repo_id, path)| (repo_id, path));
    let moved_repo_files: Vec<(RepoId, EncryptedPath, EncryptedPath)> = moved_repo_files_from
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
        mutation_state
            .repo_files
            .removed_files
            .push((repo_id, path));

        repo_files_dirty = true;
    }

    for (repo_id, from_path, to_path) in moved_repo_files {
        mutation_state
            .repo_files
            .moved_files
            .push((repo_id, from_path, to_path));

        repo_files_dirty = true;
    }

    if repo_files_dirty {
        mutation_notify(store::MutationEvent::RepoFiles, state, mutation_state);
    }
}

pub fn handle_repos_mutation(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    ciphers: &HashMap<RepoId, Arc<Cipher>>,
) {
    let mut repo_files_dirty = false;

    for repo_id in mutation_state
        .repos
        .locked_repos
        .iter()
        .chain(mutation_state.repos.removed_repos.iter())
    {
        let file_id_prefix = selectors::get_file_id(&repo_id, &EncryptedPath("".into())).0;

        state
            .repo_files
            .files
            .retain(|key, _| !key.0.starts_with(&file_id_prefix));

        state
            .repo_files
            .children
            .retain(|key, _| !key.0.starts_with(&file_id_prefix));

        state
            .repo_files
            .loaded_roots
            .retain(|key| !key.0.starts_with(&file_id_prefix));

        repo_files_dirty = true;
    }

    let mut files_to_decrypt = Vec::new();

    fn handle_path(
        state: &store::State,
        files_to_decrypt: &mut Vec<(MountId, RemotePath, RepoId, EncryptedPath)>,
        mount_id: &MountId,
        remote_path_lower: &RemotePathLower,
        repo_id: &RepoId,
        repo_path_len: usize,
    ) {
        for file in remote_files_selectors::select_files(state, &mount_id, remote_path_lower) {
            files_to_decrypt.push((
                file.mount_id.clone(),
                file.path.clone(),
                repo_id.to_owned(),
                EncryptedPath(file.path.0[repo_path_len..].to_owned()),
            ));

            if matches!(file.typ, RemoteFileType::Dir) {
                handle_path(
                    state,
                    files_to_decrypt,
                    &file.mount_id,
                    &file.path.to_lowercase(),
                    repo_id,
                    repo_path_len,
                )
            }
        }
    }

    for repo_id in mutation_state.repos.unlocked_repos.iter() {
        if let Some(repo) = state.repos.repos_by_id.get(repo_id) {
            files_to_decrypt.push((
                repo.mount_id.clone(),
                repo.path.clone(),
                repo.id.clone(),
                EncryptedPath("/".to_owned()),
            ));

            handle_path(
                state,
                &mut files_to_decrypt,
                &repo.mount_id,
                &repo.path.to_lowercase(),
                &repo_id,
                repo.path.0.len(),
            );
        }
    }

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

        mutation_notify(store::MutationEvent::RepoFiles, state, mutation_state);
    }
}

pub fn decrypt_files(
    state: &mut store::State,
    mount_id: &MountId,
    remote_path: &RemotePath,
    repo_id: &RepoId,
    encrypted_path: &EncryptedPath,
    cipher: &Cipher,
) -> Result<(), DecryptFilesError> {
    let root_remote_file_id =
        remote_files_selectors::get_file_id(mount_id, &remote_path.to_lowercase());

    if let Some(root_remote_file) = state.remote_files.files.get(&root_remote_file_id) {
        let root_repo_file = if encrypted_path.is_root() {
            get_root_file(repo_id, root_remote_file)
        } else {
            let encrypted_parent_path = EncryptedPath(
                path_utils::parent_path(&encrypted_path.0)
                    .unwrap()
                    .to_owned(),
            );
            let decrypted_parent_path = cipher.decrypt_path(&encrypted_parent_path);

            decrypt_file(
                repo_id,
                &encrypted_parent_path,
                &decrypted_parent_path,
                root_remote_file,
                &cipher,
            )
        };
        let root_repo_file_id = root_repo_file.id.clone();

        state
            .repo_files
            .files
            .insert(root_repo_file_id.clone(), root_repo_file);

        if let Some(remote_children_ids) = state.remote_files.children.get(&root_remote_file_id) {
            let path = cipher.decrypt_path(encrypted_path);

            let mut children = Vec::with_capacity(remote_children_ids.len());

            for remote_child in remote_children_ids
                .iter()
                .filter_map(|id| state.remote_files.files.get(id))
            {
                let repo_child =
                    decrypt_file(repo_id, encrypted_path, &path, remote_child, &cipher);

                children.push(repo_child.id.clone());

                state
                    .repo_files
                    .files
                    .insert(repo_child.id.clone(), repo_child);
            }

            let children_set: HashSet<RepoFileId> = children.clone().into_iter().collect();

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

            sort_children(state, root_repo_file_id.clone());
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
        let file_id = selectors::get_file_id(repo_id, &encrypted_path);

        state.repo_files.files.remove(&file_id);
    }

    Ok(())
}

pub fn decrypt_file(
    repo_id: &RepoId,
    encrypted_parent_path: &EncryptedPath,
    parent_path: &Result<DecryptedPath, DecryptFilenameError>,
    remote_file: &RemoteFile,
    cipher: &Cipher,
) -> RepoFile {
    let encrypted_path = EncryptedPath(path_utils::join_path_name(
        &encrypted_parent_path.0,
        &remote_file.name.0,
    ));
    let encrypted_name = EncryptedName(remote_file.name.0.clone());
    let id = selectors::get_file_id(repo_id, &encrypted_path);
    let name = match cipher.decrypt_filename(&encrypted_name) {
        Ok(name) => match name_utils::validate_name(&name.0) {
            Ok(()) => {
                let name_lower = name.to_lowercase().0;

                RepoFileName::Decrypted { name, name_lower }
            }
            Err(err) => RepoFileName::DecryptError {
                encrypted_name: EncryptedName(err.escaped_name.clone()),
                encrypted_name_lower: err.escaped_name.to_lowercase(),
                error: err.into(),
            },
        },
        Err(err) => RepoFileName::DecryptError {
            encrypted_name: EncryptedName(remote_file.name.0.clone()),
            encrypted_name_lower: remote_file.name_lower.0.clone(),
            error: err,
        },
    };
    let path = match (parent_path, &name) {
        (Ok(parent_path), RepoFileName::Decrypted { name, .. }) => RepoFilePath::Decrypted {
            path: repo_path_utils::join_path_name(parent_path, &name),
        },
        (Err(err), _) => RepoFilePath::DecryptError { error: err.clone() },
        (_, RepoFileName::DecryptError { error, .. }) => RepoFilePath::DecryptError {
            error: error.clone(),
        },
    };
    let size = remote_file.size.map(|size| match decrypt_size(size) {
        Ok(size) => RepoFileSize::Decrypted { size },
        Err(err) => RepoFileSize::DecryptError {
            encrypted_size: size,
            error: err,
        },
    });
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
        encrypted_path,
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

pub fn get_root_file(repo_id: &RepoId, remote_file: &RemoteFile) -> RepoFile {
    let unique_name = selectors::get_file_unique_name(&remote_file.unique_id, None);

    RepoFile {
        id: selectors::get_file_id(repo_id, &ENCRYPTED_PATH_ROOT),
        mount_id: remote_file.mount_id.clone(),
        remote_path: remote_file.path.clone(),
        repo_id: repo_id.to_owned(),
        encrypted_path: EncryptedPath("/".into()),
        path: RepoFilePath::Decrypted {
            path: DecryptedPath("/".into()),
        },
        name: RepoFileName::Decrypted {
            name: DecryptedName("".into()),
            name_lower: "".into(),
        },
        ext: None,
        content_type: None,
        typ: super::state::RepoFileType::Dir,
        size: None,
        modified: None,
        unique_name,
        remote_hash: None,
        category: FileCategory::Folder,
    }
}

pub fn cleanup_file(state: &mut store::State, file_id: &RepoFileId) {
    state.repo_files.files.remove(file_id);

    let file_id_prefix = if file_id.0.ends_with('/') {
        file_id.0.clone()
    } else {
        format!("{}/", file_id.0)
    };

    state
        .repo_files
        .files
        .retain(|file_id, _| !file_id.0.starts_with(&file_id_prefix));

    state.repo_files.children.remove(file_id);

    state
        .repo_files
        .children
        .retain(|file_id, _| !file_id.0.starts_with(&file_id_prefix));
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::{
        cipher::{
            errors::{DecryptFilenameError, DecryptSizeError},
            test_helpers::create_cipher,
        },
        files::file_category::FileCategory,
        remote_files::test_helpers as remote_files_test_helpers,
        repo_files::state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType},
        types::{DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, RepoFileId, RepoId},
    };

    use super::{decrypt_file, get_root_file};

    #[test]
    fn test_get_root_file() {
        let remote_file = remote_files_test_helpers::create_dir("m1", "/Vault");

        assert_eq!(
            get_root_file(&RepoId("r1".into()), &remote_file),
            RepoFile {
                id: RepoFileId("r1:/".into()),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
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
            &format!(
                "/Vault/{}",
                cipher.encrypt_filename(&DecryptedName("D1".into())).0
            ),
        );

        assert_eq!(
            decrypt_file(
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                &remote_file,
                &cipher
            ),
            RepoFile {
                id: RepoFileId(format!(
                    "r1:/{}",
                    cipher.encrypt_filename(&DecryptedName("D1".into())).0
                )),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
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
            decrypt_file(
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                &remote_file,
                &cipher
            ),
            RepoFile {
                id: RepoFileId("r1:/D1".into()),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: RepoId("r1".into()),
                encrypted_path: EncryptedPath(format!("/{}", remote_file.name.0)),
                path: RepoFilePath::DecryptError {
                    error: DecryptFilenameError::DecodeError(String::from(
                        "non-zero trailing bits at 1"
                    )),
                },
                name: RepoFileName::DecryptError {
                    encrypted_name: EncryptedName("D1".into()),
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
            }
        )
    }

    #[test]
    fn test_decrypt_file_file() {
        let cipher = create_cipher();
        let remote_file = remote_files_test_helpers::create_file(
            "m1",
            &format!(
                "/Vault/{}",
                cipher
                    .encrypt_filename(&DecryptedName("Image.JPG".into()))
                    .0
            ),
        );

        assert_eq!(
            decrypt_file(
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                &remote_file,
                &cipher
            ),
            RepoFile {
                id: RepoFileId(format!(
                    "r1:/{}",
                    cipher
                        .encrypt_filename(&DecryptedName("Image.JPG".into()))
                        .0
                )),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: RepoId("r1".into()),
                encrypted_path: EncryptedPath(format!("/{}", remote_file.name.0)),
                path: RepoFilePath::Decrypted {
                    path: DecryptedPath("/Image.JPG".into())
                },
                name: RepoFileName::Decrypted {
                    name: DecryptedName("Image.JPG".into()),
                    name_lower: String::from("image.jpg")
                },
                ext: Some(String::from("jpg")),
                content_type: Some(String::from("image/jpeg")),
                typ: RepoFileType::File,
                size: Some(RepoFileSize::Decrypted { size: 52 }),
                modified: Some(1),
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
        remote_file.size = Some(10);

        assert_eq!(
            decrypt_file(
                &RepoId("r1".into()),
                &EncryptedPath("/".into()),
                &Ok(DecryptedPath("/".into())),
                &remote_file,
                &cipher
            ),
            RepoFile {
                id: RepoFileId("r1:/F1".into()),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: RepoId("r1".into()),
                encrypted_path: EncryptedPath(format!("/{}", remote_file.name.0)),
                path: RepoFilePath::DecryptError {
                    error: DecryptFilenameError::DecodeError(String::from(
                        "non-zero trailing bits at 1"
                    )),
                },
                name: RepoFileName::DecryptError {
                    encrypted_name: EncryptedName("F1".into()),
                    encrypted_name_lower: String::from("f1"),
                    error: DecryptFilenameError::DecodeError(String::from(
                        "non-zero trailing bits at 1"
                    )),
                },
                ext: None,
                content_type: None,
                typ: RepoFileType::File,
                size: Some(RepoFileSize::DecryptError {
                    encrypted_size: 10,
                    error: DecryptSizeError::EncryptedFileTooShort
                }),
                modified: Some(1),
                unique_name: String::from("de40e3afb025fe16012fd421e246c711"),
                remote_hash: Some(String::from("hash")),
                category: FileCategory::Generic,
            }
        )
    }

    #[test]
    fn test_decrypt_file_parent_path_error() {
        let cipher = create_cipher();
        let remote_file = remote_files_test_helpers::create_file(
            "m1",
            &format!(
                "/Vault/dir/{}",
                cipher.encrypt_filename(&DecryptedName("F1".into())).0
            ),
        );

        assert_eq!(
            decrypt_file(
                &RepoId("r1".into()),
                &EncryptedPath("/dir".into()),
                &Err(DecryptFilenameError::DecodeError(String::from(
                    "non-zero trailing bits at 1"
                ))),
                &remote_file,
                &cipher
            ),
            RepoFile {
                id: RepoFileId(format!(
                    "r1:/dir/{}",
                    cipher.encrypt_filename(&DecryptedName("F1".into())).0
                )),
                mount_id: remote_file.mount_id.clone(),
                remote_path: remote_file.path.clone(),
                repo_id: RepoId("r1".into()),
                encrypted_path: EncryptedPath(format!(
                    "/dir/{}",
                    cipher.encrypt_filename(&DecryptedName("F1".into())).0
                )),
                path: RepoFilePath::DecryptError {
                    error: DecryptFilenameError::DecodeError(String::from(
                        "non-zero trailing bits at 1"
                    )),
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
                unique_name: String::from("2516f2ba5aeeaa8479cd8db7070ff615"),
                remote_hash: Some(String::from("hash")),
                category: FileCategory::Generic,
            }
        )
    }
}
