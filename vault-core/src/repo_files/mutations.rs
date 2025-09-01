use std::collections::{HashMap, HashSet};

use vault_crypto::data_cipher::decrypt_size;

use crate::{
    cipher::{
        Cipher,
        errors::{DecryptFilenameError, DecryptSizeError},
    },
    files::file_category::FileCategory,
    remote_files::{
        selectors as remote_files_selectors,
        state::{RemoteFile, RemoteFileType, RemoteFilesState},
    },
    repo_files::state::{RepoFilesSort, RepoFilesSortField, RepoFilesState},
    repo_files_tags::mutations::decrypt_tags,
    repos::{self, state::RepoIdNameRef},
    sort::state::{SortDirection, SortGrouping},
    store,
    types::{
        DecryptedPath, ENCRYPTED_PATH_ROOT, EncryptedName, EncryptedPath, MountId, RemotePath,
        RemotePathLower, RepoFileId, RepoId,
    },
    utils::{path_utils, remote_path_utils, repo_encrypted_path_utils, repo_path_utils},
};

use super::{
    selectors,
    state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize},
};

const SORT_CHILDREN_DEFAULT_SORT: RepoFilesSort = RepoFilesSort {
    field: RepoFilesSortField::Name,
    direction: SortDirection::Asc,
    grouping: SortGrouping::DirsFirst,
};

pub fn sort_children(repo_files: &mut RepoFilesState, file_id: RepoFileId) {
    if let Some(children_ids) = repo_files.children.get(&file_id) {
        repo_files.children.insert(
            file_id,
            selectors::select_sorted_files(
                &repo_files.files,
                children_ids,
                &SORT_CHILDREN_DEFAULT_SORT,
            ),
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
    let remote_tags_updated = mutation_state
        .remote_files
        .tags_updated
        .iter()
        .map(|(mount_id, path)| (mount_id, path.to_owned()));
    let remote_tags_updated_parents =
        mutation_state
            .remote_files
            .tags_updated
            .iter()
            .filter_map(|(mount_id, path)| {
                remote_path_utils::parent_path(path).map(|parent_path| (mount_id, parent_path))
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
                .chain(remote_moved_to_files_parents)
                .chain(remote_tags_updated)
                .chain(remote_tags_updated_parents),
        )
        .collect();

    let mut repo_files_dirty = false;

    let mut touched_repo_ids = HashSet::new();

    let mut parents_to_new_children: HashMap<RepoFileId, HashSet<RepoFileId>> = HashMap::new();

    for (mount_id, remote_path, repo_id, path) in files_to_decrypt {
        if !touched_repo_ids.contains(&repo_id) {
            touched_repo_ids.insert(repo_id.clone());
        }

        if let Some(repo) = state.repos.repos_by_id.get(&repo_id) {
            if let Ok(cipher) = repos::selectors::select_cipher_owned(state, &repo_id) {
                decrypt_files(
                    &state.remote_files,
                    &mut state.repo_files,
                    &mount_id,
                    &remote_path,
                    repo.get_id_name_ref(),
                    &path,
                    &cipher,
                );

                if let Some(parent_path) = repo_encrypted_path_utils::parent_path(&path) {
                    let parent_id = selectors::get_file_id(&repo_id, &parent_path);
                    let file_id = selectors::get_file_id(&repo_id, &path);

                    parents_to_new_children
                        .entry(parent_id)
                        .or_default()
                        .insert(file_id);
                }

                repo_files_dirty = true;
            }
        }
    }

    // Add the decrypted files to their parents' children

    for (parent_id, mut new_children) in parents_to_new_children {
        let new_sorted_children = match state.repo_files.children.get_mut(&parent_id) {
            Some(children) => {
                for child in children.iter() {
                    new_children.remove(child);
                }
                for child in new_children {
                    children.push(child);
                }

                selectors::select_sorted_files(
                    &state.repo_files.files,
                    children,
                    &SORT_CHILDREN_DEFAULT_SORT,
                )
            }
            None => selectors::select_sorted_files(
                &state.repo_files.files,
                new_children.iter(),
                &SORT_CHILDREN_DEFAULT_SORT,
            ),
        };

        state
            .repo_files
            .children
            .insert(parent_id, new_sorted_children);
    }

    for (_, _, repo_id, path) in remote_files_to_repo_files(
        state,
        mutation_state
            .remote_files
            .loaded_recent
            .iter()
            .map(|(mount_id, path)| (mount_id, path.to_owned())),
    )
    .collect::<Vec<_>>()
    {
        if path.is_root() {
            if decrypt_recent_files_repo(state, &repo_id) {
                repo_files_dirty = true;
            }
        }
    }

    for repo_id in touched_repo_ids {
        if decrypt_recent_files_repo(state, &repo_id) {
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
) {
    let mut repo_files_dirty = false;

    // Remove repo files (files, children, loaded_roots, recent) for locked or
    // removed repos

    for repo_id in mutation_state
        .repos
        .locked_repos
        .iter()
        .map(|(repo_id, _)| repo_id)
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

        state.repo_files.recent.retain(|key, _| key != repo_id);

        repo_files_dirty = true;
    }

    // Decrypt recent for unlocked repos

    for (repo_id, _) in mutation_state.repos.unlocked_repos.iter() {
        if decrypt_recent_files_repo(state, repo_id) {
            repo_files_dirty = true;
        }
    }

    // Decrypt files for unlocked repos

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

    // Recursively collect repo files to for each unlocked repo

    for (repo_id, _) in mutation_state.repos.unlocked_repos.iter() {
        if let Some(repo) = state.repos.repos_by_id.get(repo_id) {
            files_to_decrypt.push((
                repo.mount_id.clone(),
                repo.path.clone(),
                repo.id.clone(),
                EncryptedPath("/".to_owned()),
            ));

            // Recursively collect repo files to decrypt from repo root
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

    // Decrypt the collected repo files.

    for (mount_id, remote_path, repo_id, path) in files_to_decrypt {
        if let Some(repo) = state.repos.repos_by_id.get(&repo_id) {
            if let Ok(cipher) = repos::selectors::select_cipher_owned(state, &repo_id) {
                decrypt_files(
                    &state.remote_files,
                    &mut state.repo_files,
                    &mount_id,
                    &remote_path,
                    repo.get_id_name_ref(),
                    &path,
                    &cipher,
                );

                repo_files_dirty = true;
            }
        }
    }

    if repo_files_dirty {
        notify(store::Event::RepoFiles);

        mutation_notify(store::MutationEvent::RepoFiles, state, mutation_state);
    }
}

pub fn decrypt_files(
    remote_files: &RemoteFilesState,
    repo_files: &mut RepoFilesState,
    mount_id: &MountId,
    remote_path: &RemotePath,
    repo: RepoIdNameRef,
    encrypted_path: &EncryptedPath,
    cipher: &Cipher,
) {
    let root_remote_file_id =
        remote_files_selectors::get_file_id(mount_id, &remote_path.to_lowercase());

    if let Some(root_remote_file) = remote_files.files.get(&root_remote_file_id) {
        let root_repo_file = decrypt_file_path(repo, encrypted_path, root_remote_file, cipher);
        let root_repo_file_id = root_repo_file.id.clone();

        repo_files
            .files
            .insert(root_repo_file_id.clone(), root_repo_file);

        if let Some(remote_children_ids) = remote_files.children.get(&root_remote_file_id) {
            let path = cipher.decrypt_path(encrypted_path);

            let mut children = Vec::with_capacity(remote_children_ids.len());

            for remote_child in remote_children_ids
                .iter()
                .filter_map(|id| remote_files.files.get(id))
            {
                let repo_child = decrypt_file(repo, encrypted_path, &path, remote_child, &cipher);

                children.push(repo_child.id.clone());

                repo_files.files.insert(repo_child.id.clone(), repo_child);
            }

            let children_set: HashSet<RepoFileId> = children.clone().into_iter().collect();

            if let Some(old_children) = repo_files.children.get(&root_repo_file_id) {
                let old_children = old_children.clone();

                for old_child in old_children {
                    if !children_set.contains(&old_child) {
                        cleanup_file(repo_files, &old_child);
                    }
                }
            }

            repo_files
                .children
                .insert(root_repo_file_id.clone(), children);

            sort_children(repo_files, root_repo_file_id.clone());
        }

        if remote_files.loaded_roots.contains(&root_remote_file_id) {
            repo_files.loaded_roots.insert(root_repo_file_id.clone());
        }
    } else {
        let file_id = selectors::get_file_id(&repo.id, &encrypted_path);

        repo_files.files.remove(&file_id);
    }
}

pub fn decrypt_file_path(
    repo: RepoIdNameRef,
    encrypted_path: &EncryptedPath,
    remote_file: &RemoteFile,
    cipher: &Cipher,
) -> RepoFile {
    if encrypted_path.is_root() {
        get_root_file(repo, remote_file)
    } else {
        let encrypted_parent_path = EncryptedPath(
            path_utils::parent_path(&encrypted_path.0)
                .unwrap()
                .to_owned(),
        );
        let decrypted_parent_path = cipher.decrypt_path(&encrypted_parent_path);

        decrypt_file(
            repo,
            &encrypted_parent_path,
            &decrypted_parent_path,
            remote_file,
            &cipher,
        )
    }
}

pub fn decrypt_file(
    repo: RepoIdNameRef,
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
    let id = selectors::get_file_id(&repo.id, &encrypted_path);
    let name = match cipher.decrypt_filename(&encrypted_name) {
        Ok(name) => {
            let name_lower = name.to_lowercase().0;

            RepoFileName::Decrypted { name, name_lower }
        }
        Err(DecryptFilenameError::DecryptFilenameError(err)) => RepoFileName::DecryptError {
            encrypted_name: EncryptedName(remote_file.name.0.clone()),
            encrypted_name_lower: remote_file.name_lower.0.clone(),
            error: DecryptFilenameError::DecryptFilenameError(err),
        },
        Err(DecryptFilenameError::InvalidNameError(err)) => RepoFileName::DecryptError {
            encrypted_name: EncryptedName(err.escaped_name.clone()),
            encrypted_name_lower: err.escaped_name.to_lowercase(),
            error: DecryptFilenameError::InvalidNameError(err),
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
            error: DecryptSizeError::DecryptSizeError(err),
        },
    });
    let tags = decrypt_tags(remote_file, cipher);
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
        repo_id: repo.id.clone(),
        encrypted_path,
        path,
        name,
        ext,
        content_type,
        typ: (&remote_file.typ).into(),
        size,
        modified: remote_file.modified,
        tags,
        unique_name,
        remote_hash: remote_file.hash.clone(),
        category,
    }
}

pub fn decrypt_recent_files_repo(state: &mut store::State, repo_id: &RepoId) -> bool {
    let mut decrypted = false;

    if let Some(repo) = state.repos.repos_by_id.get(repo_id) {
        if let Ok(cipher) = repos::selectors::select_cipher_owned(state, &repo_id) {
            if let Some(remote_file_ids) = remote_files_selectors::select_recent(
                state,
                &repo.mount_id,
                &repo.path.to_lowercase(),
            )
            .cloned()
            {
                let repo_path_len = repo.path.0.len();

                let mut repo_file_ids = Vec::new();

                for remote_file_id in remote_file_ids {
                    if let Some(remote_file) = state.remote_files.files.get(&remote_file_id) {
                        let encrypted_path = if remote_file.path.0.len() == repo_path_len {
                            EncryptedPath("/".to_owned())
                        } else {
                            EncryptedPath(remote_file.path.0[repo_path_len..].to_owned())
                        };
                        let repo_file = decrypt_file_path(
                            repo.get_id_name_ref(),
                            &encrypted_path,
                            remote_file,
                            &cipher,
                        );
                        let repo_file_id = repo_file.id.clone();

                        state
                            .repo_files
                            .files
                            .insert(repo_file_id.clone(), repo_file);

                        repo_file_ids.push(repo_file_id);
                    }
                }

                state
                    .repo_files
                    .recent
                    .insert(repo_id.clone(), repo_file_ids);

                decrypted = true;
            }
        }
    }

    decrypted
}

pub fn get_root_file(repo: RepoIdNameRef, remote_file: &RemoteFile) -> RepoFile {
    let unique_name = selectors::get_file_unique_name(&remote_file.unique_id, None);
    let name = repo.name.clone();
    let name_lower = name.to_lowercase().0;

    RepoFile {
        id: selectors::get_file_id(&repo.id, &ENCRYPTED_PATH_ROOT),
        mount_id: remote_file.mount_id.clone(),
        remote_path: remote_file.path.clone(),
        repo_id: repo.id.clone(),
        encrypted_path: EncryptedPath("/".into()),
        path: RepoFilePath::Decrypted {
            path: DecryptedPath("/".into()),
        },
        name: RepoFileName::Decrypted { name, name_lower },
        ext: None,
        content_type: None,
        typ: super::state::RepoFileType::Dir,
        size: None,
        modified: Some(*repo.added),
        tags: None,
        unique_name,
        remote_hash: None,
        category: FileCategory::Folder,
    }
}

pub fn cleanup_file(repo_files: &mut RepoFilesState, file_id: &RepoFileId) {
    repo_files.files.remove(file_id);

    let file_id_prefix = if file_id.0.ends_with('/') {
        file_id.0.clone()
    } else {
        format!("{}/", file_id.0)
    };

    repo_files
        .files
        .retain(|file_id, _| !file_id.0.starts_with(&file_id_prefix));

    repo_files.children.remove(file_id);

    repo_files
        .children
        .retain(|file_id, _| !file_id.0.starts_with(&file_id_prefix));
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use similar_asserts::assert_eq;

    use crate::{
        cipher::{
            Cipher,
            errors::{DecryptFilenameError, DecryptSizeError},
            test_helpers::create_cipher,
        },
        files::file_category::FileCategory,
        remote_files::test_helpers as remote_files_test_helpers,
        repo_files::state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType},
        repos::state::{Repo, RepoState},
        types::{
            DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, MountId, RemotePath,
            RepoFileId, RepoId,
        },
    };

    use super::{decrypt_file, get_root_file};

    fn create_dummy_repo(cipher: Arc<Cipher>) -> Repo {
        Repo {
            id: RepoId("r1".into()),
            name: DecryptedName("My safe box".into()),
            mount_id: MountId("m1".into()),
            path: RemotePath("/Vault".into()),
            salt: None,
            added: 3,
            password_validator: "".into(),
            password_validator_encrypted: "".into(),
            state: RepoState::Unlocked { cipher },
            web_url: "".into(),
            last_activity: None,
            auto_lock: None,
        }
    }

    #[test]
    fn test_get_root_file() {
        let cipher = Arc::new(create_cipher());
        let repo = create_dummy_repo(cipher.clone());
        let remote_file = remote_files_test_helpers::create_dir("m1", "/Vault");

        assert_eq!(
            get_root_file(repo.get_id_name_ref(), &remote_file),
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
                    name: DecryptedName("My safe box".into()),
                    name_lower: String::from("my safe box")
                },
                ext: None,
                content_type: None,
                typ: RepoFileType::Dir,
                size: None,
                modified: Some(3),
                tags: None,
                unique_name: String::from("b3278dc2a959498ee943d7dbe02ae093"),
                remote_hash: None,
                category: FileCategory::Folder,
            }
        )
    }

    #[test]
    fn test_decrypt_file_dir() {
        let cipher = Arc::new(create_cipher());
        let repo = create_dummy_repo(cipher.clone());
        let remote_file = remote_files_test_helpers::create_dir(
            "m1",
            &format!(
                "/Vault/{}",
                cipher.encrypt_filename(&DecryptedName("D1".into())).0
            ),
        );

        assert_eq!(
            decrypt_file(
                repo.get_id_name_ref(),
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
                modified: Some(1),
                tags: None,
                unique_name: String::from("0d54d1cb1830c80318d2bde653bda396"),
                remote_hash: None,
                category: FileCategory::Folder,
            }
        )
    }

    #[test]
    fn test_decrypt_file_dir_decrypt_error() {
        let cipher = Arc::new(create_cipher());
        let repo = create_dummy_repo(cipher.clone());
        let remote_file = remote_files_test_helpers::create_dir("m1", "/Vault/D1");

        assert_eq!(
            decrypt_file(
                repo.get_id_name_ref(),
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
                modified: Some(1),
                tags: None,
                unique_name: String::from("f8a02d0ff4cb571f2c17c05dc8bdf626"),
                remote_hash: None,
                category: FileCategory::Folder,
            }
        )
    }

    #[test]
    fn test_decrypt_file_file() {
        let cipher = Arc::new(create_cipher());
        let repo = create_dummy_repo(cipher.clone());
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
                repo.get_id_name_ref(),
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
                tags: None,
                unique_name: String::from("c7f010983b2f25f3e1d604c2870d82c8.jpg"),
                remote_hash: Some(String::from("hash")),
                category: FileCategory::Image,
            }
        )
    }

    #[test]
    fn test_decrypt_file_file_decrypt_error() {
        let cipher = Arc::new(create_cipher());
        let repo = create_dummy_repo(cipher.clone());
        let mut remote_file = remote_files_test_helpers::create_file("m1", "/Vault/F1");
        remote_file.size = Some(10);

        assert_eq!(
            decrypt_file(
                repo.get_id_name_ref(),
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
                size: Some(RepoFileSize::DecryptError {
                    encrypted_size: 10,
                    error: DecryptSizeError::DecryptSizeError(
                        vault_crypto::errors::DecryptSizeError::EncryptedFileTooShort
                    )
                }),
                modified: Some(1),
                tags: None,
                unique_name: String::from("de40e3afb025fe16012fd421e246c711"),
                remote_hash: Some(String::from("hash")),
                category: FileCategory::Generic,
            }
        )
    }

    #[test]
    fn test_decrypt_file_parent_path_error() {
        let cipher = Arc::new(create_cipher());
        let repo = create_dummy_repo(cipher.clone());
        let remote_file = remote_files_test_helpers::create_file(
            "m1",
            &format!(
                "/Vault/dir/{}",
                cipher.encrypt_filename(&DecryptedName("F1".into())).0
            ),
        );

        assert_eq!(
            decrypt_file(
                repo.get_id_name_ref(),
                &EncryptedPath("/dir".into()),
                &Err(DecryptFilenameError::DecryptFilenameError(
                    vault_crypto::errors::DecryptFilenameError::DecodeError(
                        "non-zero trailing bits at 1".into()
                    )
                )),
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
                unique_name: String::from("2516f2ba5aeeaa8479cd8db7070ff615"),
                remote_hash: Some(String::from("hash")),
                category: FileCategory::Generic,
            }
        )
    }
}
