use futures::{io::Cursor, FutureExt};
use similar_asserts::assert_eq;
use vault_core::{
    cipher::errors::DecryptFilenameError,
    common::errors::InvalidNameError,
    files::file_category::FileCategory,
    repo_files::state::{
        RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType, RepoFilesState,
        RepoFilesUploadConflictResolution,
    },
    types::{DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, RemotePath, RepoFileId},
};
use vault_core_tests::helpers::with_repo;

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

            fixture.unlock().await;

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
                        parent_path: DecryptedPath("/".into()),
                        encrypted_name: EncryptedName("Plain.txt".into()),
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
                            parent_path: DecryptedPath("/".into()),
                            encrypted_name: EncryptedName("Plain.txt".into()),
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
            fixture
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
                .await
                .unwrap();

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
                        parent_path: DecryptedPath("/".into()),
                        encrypted_name: EncryptedName("A\\n/\\n".into()),
                        error: DecryptFilenameError::InvalidName(InvalidNameError::new("A\n/\n")),
                    },
                    name: RepoFileName::DecryptError {
                        encrypted_name: EncryptedName("A\\n/\\n".into()),
                        encrypted_name_lower: "a\\n/\\n".into(),
                        error: DecryptFilenameError::InvalidName(InvalidNameError::new("A\n/\n")),
                    },
                    ext: None,
                    content_type: None,
                    typ: RepoFileType::File,
                    size: Some(file.size.clone().unwrap()),
                    modified: Some(file.modified.unwrap()),
                    unique_name: file.unique_name.clone(),
                    remote_hash: Some(file.remote_hash.clone().unwrap()),
                    category: FileCategory::Generic,
                }
            );
        }
        .boxed()
    });
}
