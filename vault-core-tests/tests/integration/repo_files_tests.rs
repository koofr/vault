use std::collections::HashMap;

use futures::{FutureExt, io::Cursor, join};
use similar_asserts::assert_eq;
use vault_core::{
    cipher::errors::DecryptFilenameError,
    common::errors::InvalidNameError,
    files::file_category::FileCategory,
    repo_files::state::{
        RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType, RepoFilesState,
        RepoFilesUploadConflictResolution,
    },
    store,
    types::{DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, RemotePath, RepoFileId},
};
use vault_core_tests::helpers::{eventstream::eventstream_subscribe, with_repo};

#[test]
fn test_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            let get_state = || fixture.vault.with_state(|state| state.repo_files.clone());

            let _ = fixture.upload_file("/file1.txt", "test").await;
            fixture.create_dir("/dir1").await;
            let _ = fixture.upload_file("/dir1/file11.txt", "test").await;
            fixture.create_dir("/dir1/dir12").await;
            let _ = fixture.upload_file("/dir1/dir12/file121.txt", "test").await;

            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, &EncryptedPath("/".into()))
                .await
                .unwrap();
            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, &fixture.encrypt_path("/dir1".into()))
                .await
                .unwrap();
            fixture
                .vault
                .repo_files_service
                .load_files(
                    &fixture.repo_id,
                    &fixture.encrypt_path("/dir1/dir12".into()),
                )
                .await
                .unwrap();

            let state_before_lock = get_state();

            fixture.lock();

            let state_after_lock = get_state();

            assert_eq!(state_after_lock, RepoFilesState::default());

            fixture.unlock();

            let state_after_unlock = get_state();

            assert_eq!(state_after_unlock, state_before_lock);

            fixture.remove().await;

            let state_after_remove = get_state();

            assert_eq!(state_after_remove, RepoFilesState::default());
        }
        .boxed()
    });
}

#[test]
fn test_recent_file_deleted() {
    with_repo(|fixture| {
        async move {
            fixture.create_dir("/dir1").await;
            fixture.upload_file("/dir1/file11.txt", "test").await;
            fixture.create_dir("/dir1/dir12").await;
            fixture.upload_file("/dir1/dir12/file121.txt", "test").await;
            fixture.create_dir("/dir12").await;
            fixture.upload_file("/dir12/file121.txt", "test").await;
            fixture.upload_file("/file.txt", "test").await;

            fixture
                .vault
                .repo_files_service
                .load_recent(&fixture.repo_id, 1000, None)
                .await
                .unwrap();

            assert_eq!(
                fixture.get_recent(),
                HashMap::from([(
                    fixture.repo_id.clone(),
                    vec![
                        fixture.get_file_id("/file.txt"),
                        fixture.get_file_id("/dir12/file121.txt"),
                        fixture.get_file_id("/dir12"),
                        fixture.get_file_id("/dir1/dir12/file121.txt"),
                        fixture.get_file_id("/dir1/dir12"),
                        fixture.get_file_id("/dir1/file11.txt"),
                        fixture.get_file_id("/dir1"),
                        fixture.get_file_id("/"),
                    ]
                )])
            );

            {
                let (mount_id, remote_path) = fixture
                    .vault
                    .repo_files_service
                    .get_repo_mount_path(&fixture.repo_id, &fixture.encrypt_path("/dir1"))
                    .unwrap();

                fixture
                    .vault
                    .remote_files_service
                    .delete_file(&mount_id, &remote_path)
                    .await
                    .unwrap();
            }

            assert_eq!(
                fixture.get_recent(),
                HashMap::from([(
                    fixture.repo_id.clone(),
                    vec![
                        fixture.get_file_id("/file.txt"),
                        fixture.get_file_id("/dir12/file121.txt"),
                        fixture.get_file_id("/dir12"),
                        fixture.get_file_id("/"),
                    ]
                )])
            );
        }
        .boxed()
    });
}

#[test]
fn test_recent_file_moved() {
    with_repo(|fixture| {
        async move {
            fixture.create_dir("/dir1").await;
            fixture.upload_file("/dir1/file11.txt", "test").await;
            fixture.create_dir("/dir1/dir12").await;
            fixture.upload_file("/dir1/dir12/file121.txt", "test").await;
            fixture.create_dir("/dir12").await;
            fixture.upload_file("/dir12/file121.txt", "test").await;
            fixture.upload_file("/file.txt", "test").await;

            fixture
                .vault
                .repo_files_service
                .load_recent(&fixture.repo_id, 1000, None)
                .await
                .unwrap();

            assert_eq!(
                fixture.get_recent(),
                HashMap::from([(
                    fixture.repo_id.clone(),
                    vec![
                        fixture.get_file_id("/file.txt"),
                        fixture.get_file_id("/dir12/file121.txt"),
                        fixture.get_file_id("/dir12"),
                        fixture.get_file_id("/dir1/dir12/file121.txt"),
                        fixture.get_file_id("/dir1/dir12"),
                        fixture.get_file_id("/dir1/file11.txt"),
                        fixture.get_file_id("/dir1"),
                        fixture.get_file_id("/"),
                    ]
                )])
            );

            let eventstream_subscription = eventstream_subscribe(
                fixture.vault.store.clone(),
                fixture.mount_id.clone(),
                RemotePath("/".into()),
                "test",
            )
            .await;

            let move_future = async {
                let (mount_id, remote_path) = fixture
                    .vault
                    .repo_files_service
                    .get_repo_mount_path(&fixture.repo_id, &fixture.encrypt_path("/dir1"))
                    .unwrap();
                let (_, to_remote_path) = fixture
                    .vault
                    .repo_files_service
                    .get_repo_mount_path(&fixture.repo_id, &fixture.encrypt_path("/dir12/dirx"))
                    .unwrap();

                fixture
                    .vault
                    .remote_files_service
                    .move_file(&mount_id, &remote_path, &mount_id, &to_remote_path)
                    .await
                    .unwrap()
            };
            let moved_future = store::wait_for(
                fixture.vault.store.clone(),
                &[store::Event::RemoteFiles],
                move |mutation_state| {
                    mutation_state
                        .filter(|state| !state.remote_files.moved_files.is_empty())
                        .map(|_| ())
                },
            );
            let _ = join!(move_future, moved_future);

            drop(eventstream_subscription);

            assert_eq!(
                fixture.get_recent(),
                HashMap::from([(
                    fixture.repo_id.clone(),
                    vec![
                        fixture.get_file_id("/file.txt"),
                        fixture.get_file_id("/dir12/file121.txt"),
                        fixture.get_file_id("/dir12"),
                        fixture.get_file_id("/dir12/dirx/dir12/file121.txt"),
                        fixture.get_file_id("/dir12/dirx/dir12"),
                        fixture.get_file_id("/dir12/dirx/file11.txt"),
                        fixture.get_file_id("/dir12/dirx"),
                        fixture.get_file_id("/"),
                    ]
                )])
            );
        }
        .boxed()
    });
}

#[test]
fn test_recent_repo_lock_unlock_remove() {
    with_repo(|fixture| {
        async move {
            let get_state = || fixture.vault.with_state(|state| state.repo_files.clone());

            fixture.create_dir("/dir1").await;
            fixture.upload_file("/dir1/file11.txt", "test").await;
            fixture.create_dir("/dir1/dir12").await;
            fixture.upload_file("/dir1/dir12/file121.txt", "test").await;
            fixture.create_dir("/dir12").await;
            fixture.upload_file("/dir12/file121.txt", "test").await;
            fixture.upload_file("/file.txt", "test").await;

            fixture
                .vault
                .repo_files_service
                .load_recent(&fixture.repo_id, 1000, None)
                .await
                .unwrap();

            let state_before_lock = get_state();

            fixture.lock();

            let state_after_lock = get_state();

            assert_eq!(state_after_lock, RepoFilesState::default());

            fixture.unlock();

            let state_after_unlock = get_state();

            assert_eq!(state_after_unlock, state_before_lock);

            fixture.remove().await;

            let state_after_remove = get_state();

            assert_eq!(state_after_remove, RepoFilesState::default());
        }
        .boxed()
    });
}

#[test]
fn test_name_decryption_error() {
    with_repo(|fixture| {
        async move {
            fixture
                .user_fixture
                .upload_remote_file("/My safe box/Plain.txt", "test")
                .await;

            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, &EncryptedPath("/".into()))
                .await
                .unwrap();

            let file = fixture.vault.with_state(|state| {
                vault_core::repo_files::selectors::select_files(
                    state,
                    &fixture.repo_id,
                    &EncryptedPath("/".into()),
                )
                .next()
                .cloned()
                .unwrap()
            });

            assert_eq!(
                file,
                RepoFile {
                    id: RepoFileId(format!("{}:/Plain.txt", fixture.repo_id.0)),
                    mount_id: fixture.mount_id.clone(),
                    remote_path: RemotePath("/My safe box/Plain.txt".into()),
                    repo_id: fixture.repo_id.clone(),
                    encrypted_path: EncryptedPath("/Plain.txt".into()),
                    path: RepoFilePath::DecryptError {
                        error: file.path.decrypted_path().clone().unwrap_err()
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("Plain.txt".into()),
                        encrypted_name_lower: "plain.txt".into(),
                        error: file.name.decrypted_name().clone().unwrap_err()
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(file.size.clone().unwrap()),
                    modified: Some(file.modified.unwrap()),
                    tags: None,
                    unique_name: file.unique_name.clone(),
                    remote_hash: Some(file.remote_hash.clone().unwrap()),
                    category: FileCategory::Generic,
                }
            );
        }
        .boxed()
    });
}

#[test]
fn test_encrypted_decrypted_same_name() {
    with_repo(|fixture| {
        async move {
            fixture.upload_file("/Plain.txt", "test").await;
            fixture
                .user_fixture
                .upload_remote_file("/My safe box/Plain.txt", "test")
                .await;

            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, &EncryptedPath("/".into()))
                .await
                .unwrap();

            let files = fixture.vault.with_state(|state| {
                vault_core::repo_files::selectors::select_files(
                    state,
                    &fixture.repo_id,
                    &EncryptedPath("/".into()),
                )
                .cloned()
                .collect::<Vec<_>>()
            });

            let err_file = files.get(0).cloned().unwrap();
            let ok_file = files.get(1).cloned().unwrap();

            assert_eq!(
                files,
                vec![
                    RepoFile {
                        id: RepoFileId(format!("{}:/Plain.txt", fixture.repo_id.0)),
                        mount_id: fixture.mount_id.clone(),
                        remote_path: RemotePath("/My safe box/Plain.txt".into()),
                        repo_id: fixture.repo_id.clone(),
                        encrypted_path: EncryptedPath("/Plain.txt".into()),
                        path: RepoFilePath::DecryptError {
                            error: err_file.path.decrypted_path().clone().unwrap_err()
                        },
                        name: RepoFileName::DecryptError {
                            encrypted_name: EncryptedName("Plain.txt".into()),
                            encrypted_name_lower: "plain.txt".into(),
                            error: err_file.name.decrypted_name().clone().unwrap_err()
                        },
                        ext: None,
                        content_type: None,
                        typ: RepoFileType::File,
                        size: Some(err_file.size.clone().unwrap()),
                        modified: Some(err_file.modified.unwrap()),
                        tags: err_file.tags.clone(),
                        unique_name: err_file.unique_name.clone(),
                        remote_hash: Some(err_file.remote_hash.clone().unwrap()),
                        category: FileCategory::Generic,
                    },
                    RepoFile {
                        id: RepoFileId(format!(
                            "{}:{}",
                            fixture.repo_id.0,
                            fixture.encrypt_path("/Plain.txt").0
                        )),
                        mount_id: fixture.mount_id.clone(),
                        remote_path: RemotePath(format!(
                            "/My safe box/{}",
                            fixture
                                .vault
                                .repo_files_service
                                .encrypt_filename(
                                    &fixture.repo_id,
                                    &DecryptedName("Plain.txt".into())
                                )
                                .unwrap()
                                .0
                        )),
                        repo_id: fixture.repo_id.clone(),
                        encrypted_path: EncryptedPath(format!(
                            "/{}",
                            fixture
                                .vault
                                .repo_files_service
                                .encrypt_filename(
                                    &fixture.repo_id,
                                    &DecryptedName("Plain.txt".into())
                                )
                                .unwrap()
                                .0
                        )),
                        path: RepoFilePath::Decrypted {
                            path: DecryptedPath("/Plain.txt".into())
                        },
                        name: RepoFileName::Decrypted {
                            name: DecryptedName("Plain.txt".into()),
                            name_lower: "plain.txt".into()
                        },
                        ext: Some("txt".into()),
                        content_type: Some("text/plain".into()),
                        typ: RepoFileType::File,
                        size: Some(RepoFileSize::Decrypted { size: 4 }),
                        modified: Some(ok_file.modified.unwrap()),
                        tags: ok_file.tags.clone(),
                        unique_name: ok_file.unique_name.clone(),
                        remote_hash: Some(ok_file.remote_hash.clone().unwrap()),
                        category: FileCategory::Text,
                    },
                ]
            );
        }
        .boxed()
    });
}

#[test]
fn test_invalid_name() {
    with_repo(|fixture| {
        async move {
            // upload fails because after upload it tries to decrypt the name
            // and the name is invalid
            let _ = fixture
                .vault
                .repo_files_service
                .clone()
                .upload_file_reader(
                    &fixture.repo_id,
                    &EncryptedPath("/".into()),
                    fixture.encrypt_filename("A\n/\n"),
                    Box::pin(Cursor::new("text".as_bytes().to_vec())),
                    Some(4),
                    RepoFilesUploadConflictResolution::Error,
                    None,
                )
                .await;

            fixture
                .vault
                .repo_files_service
                .load_files(&fixture.repo_id, &EncryptedPath("/".into()))
                .await
                .unwrap();

            let file = fixture.vault.with_state(|state| {
                vault_core::repo_files::selectors::select_files(
                    state,
                    &fixture.repo_id,
                    &EncryptedPath("/".into()),
                )
                .next()
                .cloned()
                .unwrap()
            });

            assert_eq!(
                file,
                RepoFile {
                    id: RepoFileId(format!(
                        "{}:/{}",
                        fixture.repo_id.0,
                        fixture.encrypt_filename("A\n/\n").0
                    )),
                    mount_id: fixture.mount_id.clone(),
                    remote_path: RemotePath(format!(
                        "/My safe box/{}",
                        fixture
                            .vault
                            .repo_files_service
                            .encrypt_filename(&fixture.repo_id, &DecryptedName("A\n/\n".into()))
                            .unwrap()
                            .0
                    )),
                    repo_id: fixture.repo_id.clone(),
                    encrypted_path: EncryptedPath(format!(
                        "/{}",
                        fixture
                            .vault
                            .repo_files_service
                            .encrypt_filename(&fixture.repo_id, &DecryptedName("A\n/\n".into()))
                            .unwrap()
                            .0
                    )),
                    path: RepoFilePath::DecryptError {
                        error: DecryptFilenameError::InvalidNameError(InvalidNameError::new(
                            "A\n/\n"
                        )),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("A\\n/\\n".into()),
                        encrypted_name_lower: "a\\n/\\n".into(),
                        error: DecryptFilenameError::InvalidNameError(InvalidNameError::new(
                            "A\n/\n"
                        )),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(file.size.clone().unwrap()),
                    modified: Some(file.modified.unwrap()),
                    tags: None,
                    unique_name: file.unique_name.clone(),
                    remote_hash: Some(file.remote_hash.clone().unwrap()),
                    category: FileCategory::Generic,
                }
            );
        }
        .boxed()
    });
}
