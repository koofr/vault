use std::collections::HashMap;

use futures::{io::Cursor, FutureExt};
use similar_asserts::assert_eq;
use vault_core::{
    remote::{models, remote::RemoteFileTagsSetConditions, RemoteFileUploadConflictResolution},
    remote_files, repo_files,
    repo_files_tags::{self, errors::DecryptTagsError, state::RepoFileTags},
    types::{DecryptedName, EncryptedPath, RemoteName},
    utils::remote_path_utils,
};
use vault_core_tests::helpers::with_repo;

#[test]
fn test_upload_overwrite() {
    with_repo(|fixture| {
        async move {
            let (_, repo_file) = fixture.upload_file("/file.txt", "test").await;

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: Some(hex::decode(repo_file.remote_hash.unwrap()).unwrap()),
                    hash: Some(md5::compute("test").to_vec()),
                    unknown: HashMap::new(),
                }))
            );

            let (_, repo_file) = fixture.upload_file("/file.txt", "test1").await;

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: Some(hex::decode(repo_file.remote_hash.unwrap()).unwrap()),
                    hash: Some(md5::compute("test1").to_vec()),
                    unknown: HashMap::new(),
                }))
            );
        }
        .boxed()
    });
}

#[test]
fn test_upload_overwrite_without_hash() {
    with_repo(|fixture| {
        async move {
            let (_, repo_file) = fixture.upload_file("/file.txt", "test").await;

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: Some(
                        hex::decode(repo_file.remote_hash.clone().unwrap()).unwrap()
                    ),
                    hash: Some(md5::compute("test").to_vec()),
                    unknown: HashMap::new(),
                }))
            );

            let cipher = fixture
                .vault
                .repos_service
                .get_cipher(&fixture.repo_id)
                .unwrap();
            let (mount_id, remote_parent_path) = fixture
                .vault
                .repo_files_service
                .get_repo_mount_path(&fixture.repo_id, &EncryptedPath("/".into()))
                .unwrap();
            let encrypted_reader = cipher.encrypt_reader_async(Cursor::new("test1".as_bytes()));

            fixture
                .vault
                .remote_files_service
                .upload_file_reader(
                    &mount_id,
                    &remote_parent_path,
                    &RemoteName(cipher.encrypt_filename(&DecryptedName("file.txt".into())).0),
                    Box::pin(encrypted_reader),
                    None,
                    RemoteFileUploadConflictResolution::Overwrite {
                        if_size: None,
                        if_modified: None,
                        if_hash: None,
                        ignore_nonexisting: false,
                    },
                    None,
                )
                .await
                .unwrap();

            let old_repo_file = repo_file;
            let repo_file = fixture.vault.with_state(|state| {
                state
                    .repo_files
                    .files
                    .get(&old_repo_file.id)
                    .cloned()
                    .unwrap()
            });

            assert_eq!(
                repo_file.tags,
                Some(Err(DecryptTagsError::EncryptedHashMismatch {
                    expected_encrypted_hash: Some(old_repo_file.remote_hash.clone().unwrap()),
                    encrypted_hash: repo_file.remote_hash.clone()
                }))
            );
        }
        .boxed()
    });
}

#[test]
fn test_upload_without_hash() {
    with_repo(|fixture| {
        async move {
            let cipher = fixture
                .vault
                .repos_service
                .get_cipher(&fixture.repo_id)
                .unwrap();
            let (mount_id, remote_parent_path) = fixture
                .vault
                .repo_files_service
                .get_repo_mount_path(&fixture.repo_id, &EncryptedPath("/".into()))
                .unwrap();
            let encrypted_reader = cipher.encrypt_reader_async(Cursor::new("test".as_bytes()));

            fixture
                .vault
                .remote_files_service
                .upload_file_reader(
                    &mount_id,
                    &remote_parent_path,
                    &RemoteName(fixture.encrypt_filename("file.txt").0),
                    Box::pin(encrypted_reader),
                    None,
                    RemoteFileUploadConflictResolution::Overwrite {
                        if_size: None,
                        if_modified: None,
                        if_hash: None,
                        ignore_nonexisting: false,
                    },
                    None,
                )
                .await
                .unwrap();

            let repo_file = fixture.vault.with_state(|state| {
                state
                    .repo_files
                    .files
                    .get(&repo_files::selectors::get_file_id(
                        &fixture.repo_id,
                        &fixture.encrypt_path("/file.txt"),
                    ))
                    .cloned()
                    .unwrap()
            });

            assert_eq!(repo_file.tags, None);
        }
        .boxed()
    });
}

#[test]
fn test_only_unknown() {
    with_repo(|fixture| {
        async move {
            let cipher = fixture
                .vault
                .repos_service
                .get_cipher(&fixture.repo_id)
                .unwrap();
            let (mount_id, remote_parent_path) = fixture
                .vault
                .repo_files_service
                .get_repo_mount_path(&fixture.repo_id, &EncryptedPath("/".into()))
                .unwrap();
            let encrypted_reader = cipher.encrypt_reader_async(Cursor::new("test".as_bytes()));

            fixture
                .vault
                .remote_files_service
                .upload_file_reader(
                    &mount_id,
                    &remote_parent_path,
                    &RemoteName(fixture.encrypt_filename("file.txt").0),
                    Box::pin(encrypted_reader),
                    None,
                    RemoteFileUploadConflictResolution::Overwrite {
                        if_size: None,
                        if_modified: None,
                        if_hash: None,
                        ignore_nonexisting: false,
                    },
                    None,
                )
                .await
                .unwrap();

            let path = fixture.encrypt_path("/file.txt");

            fixture
                .vault
                .repo_files_tags_service
                .set_tags(
                    &fixture.repo_id,
                    &path,
                    Box::new(|_file, tags| {
                        tags.unknown.insert("k1".into(), "v1".into());
                        Ok(())
                    }),
                )
                .await
                .unwrap();

            let repo_file = fixture.vault.with_state(|state| {
                state
                    .repo_files
                    .files
                    .get(&repo_files::selectors::get_file_id(
                        &fixture.repo_id,
                        &fixture.encrypt_path("/file.txt"),
                    ))
                    .cloned()
                    .unwrap()
            });

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: Some(hex::decode(repo_file.remote_hash.unwrap()).unwrap()),
                    hash: None,
                    unknown: HashMap::from([("k1".into(), "v1".into())]),
                }))
            );
        }
        .boxed()
    });
}

#[test]
fn test_only_unknown_no_encrypted_hash() {
    with_repo(|fixture| {
        async move {
            let cipher = fixture
                .vault
                .repos_service
                .get_cipher(&fixture.repo_id)
                .unwrap();
            let (mount_id, remote_parent_path) = fixture
                .vault
                .repo_files_service
                .get_repo_mount_path(&fixture.repo_id, &EncryptedPath("/".into()))
                .unwrap();
            let remote_name = RemoteName(fixture.encrypt_filename("file.txt").0);
            let remote_path = remote_path_utils::join_path_name(&remote_parent_path, &remote_name);
            let encrypted_reader = cipher.encrypt_reader_async(Cursor::new("test".as_bytes()));

            fixture
                .vault
                .remote_files_service
                .upload_file_reader(
                    &mount_id,
                    &remote_parent_path,
                    &remote_name,
                    Box::pin(encrypted_reader),
                    None,
                    RemoteFileUploadConflictResolution::Overwrite {
                        if_size: None,
                        if_modified: None,
                        if_hash: None,
                        ignore_nonexisting: false,
                    },
                    None,
                )
                .await
                .unwrap();

            let tags = RepoFileTags {
                encrypted_hash: None,
                hash: None,
                unknown: HashMap::from([("k1".into(), "v1".into())]),
            };
            let encrypted_tags = tags.to_string(&cipher).unwrap();
            let remote_tags = HashMap::from([(
                repo_files_tags::selectors::REMOTE_FILE_TAGS_KEY.to_owned(),
                vec![encrypted_tags],
            )]);

            fixture
                .vault
                .remote_files_service
                .set_tags(
                    &mount_id,
                    &remote_path,
                    remote_tags,
                    RemoteFileTagsSetConditions::default(),
                )
                .await
                .unwrap();

            let repo_file = fixture.vault.with_state(|state| {
                state
                    .repo_files
                    .files
                    .get(&repo_files::selectors::get_file_id(
                        &fixture.repo_id,
                        &fixture.encrypt_path("/file.txt"),
                    ))
                    .cloned()
                    .unwrap()
            });

            assert_eq!(
                repo_file.tags,
                Some(Err(DecryptTagsError::EncryptedHashMismatch {
                    expected_encrypted_hash: None,
                    encrypted_hash: None
                }))
            );
        }
        .boxed()
    });
}

#[test]
fn test_tags_conflict_retry() {
    with_repo(|fixture| {
        async move {
            let (_, repo_file) = fixture.upload_file("/file.txt", "test").await;

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: Some(
                        hex::decode(repo_file.remote_hash.clone().unwrap()).unwrap()
                    ),
                    hash: Some(md5::compute("test").to_vec()),
                    unknown: HashMap::new(),
                }))
            );

            let path = fixture.encrypt_path("/file.txt");

            fixture
                .vault
                .store
                .mutate(|state, notify, mutation_state, mutation_notify| {
                    let remote_file =
                        repo_files::selectors::select_remote_file(state, &repo_file).unwrap();

                    remote_files::mutations::file_tags_updated(
                        state,
                        notify,
                        mutation_state,
                        mutation_notify,
                        &fixture.mount_id,
                        &repo_file.remote_path,
                        models::FilesFile {
                            name: remote_file.name.clone(),
                            typ: "file".into(),
                            modified: remote_file.modified.unwrap(),
                            size: remote_file.size.unwrap(),
                            content_type: "text/plain".into(),
                            hash: remote_file.hash.clone(),
                            tags: HashMap::from([(
                                repo_files_tags::selectors::REMOTE_FILE_TAGS_KEY.to_owned(),
                                vec!["foo".into()],
                            )]),
                        },
                    );
                });

            fixture
                .vault
                .repo_files_tags_service
                .set_tags(
                    &fixture.repo_id,
                    &path,
                    Box::new(|_file, tags| {
                        tags.unknown.insert("k1".into(), "v1".into());
                        Ok(())
                    }),
                )
                .await
                .unwrap();

            let repo_file = fixture.vault.with_state(|state| {
                state
                    .repo_files
                    .files
                    .get(&repo_files::selectors::get_file_id(
                        &fixture.repo_id,
                        &fixture.encrypt_path("/file.txt"),
                    ))
                    .cloned()
                    .unwrap()
            });

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: Some(hex::decode(repo_file.remote_hash.unwrap()).unwrap()),
                    hash: Some(md5::compute("test").to_vec()),
                    unknown: HashMap::from([("k1".into(), "v1".into())]),
                }))
            );
        }
        .boxed()
    });
}

#[test]
fn test_dir() {
    with_repo(|fixture| {
        async move {
            fixture.create_dir("/dir").await;

            let path = fixture.encrypt_path("/dir");

            fixture
                .vault
                .repo_files_tags_service
                .set_tags(
                    &fixture.repo_id,
                    &path,
                    Box::new(|_file, tags| {
                        tags.unknown.insert("k1".into(), "v1".into());
                        Ok(())
                    }),
                )
                .await
                .unwrap();

            let repo_file = fixture.vault.with_state(|state| {
                state
                    .repo_files
                    .files
                    .get(&repo_files::selectors::get_file_id(&fixture.repo_id, &path))
                    .cloned()
                    .unwrap()
            });

            assert_eq!(
                repo_file.tags,
                Some(Ok(RepoFileTags {
                    encrypted_hash: None,
                    hash: None,
                    unknown: HashMap::from([("k1".into(), "v1".into())]),
                }))
            );
        }
        .boxed()
    });
}
