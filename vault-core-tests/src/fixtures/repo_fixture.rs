use std::sync::Arc;

use futures::io::Cursor;
use vault_core::{
    cipher::Cipher,
    repo_files::{
        self,
        state::{RepoFile, RepoFilesUploadConflictResolution, RepoFilesUploadResult},
    },
    repos::state::RepoUnlockMode,
    types::{
        DecryptedName, DecryptedPath, EncryptedName, EncryptedPath, MountId, RemotePath,
        RepoFileId, RepoId,
    },
    utils::repo_encrypted_path_utils,
    Vault,
};
use vault_fake_remote::fake_remote::context::Context;

use crate::{fake_remote::FakeRemote, fixtures::user_fixture::UserFixture};

pub struct RepoFixture {
    pub user_fixture: Arc<UserFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub vault: Arc<Vault>,
    pub mount_id: MountId,
    pub path: RemotePath,
    pub repo_id: RepoId,
    pub cipher: Arc<Cipher>,
}

impl RepoFixture {
    pub async fn create(user_fixture: Arc<UserFixture>) -> Arc<Self> {
        let fake_remote = user_fixture.fake_remote.clone();
        let vault = user_fixture.vault.clone();

        let repo = fake_remote
            .app_state
            .vault_repos_create_service
            .create_test_vault_repo(&Context {
                user_id: user_fixture.user_id.clone(),
                user_agent: None,
            })
            .await
            .unwrap();

        let mount_id = repo.mount_id;
        let path = repo.path;
        let repo_id = repo.id;
        let cipher = Arc::new(Cipher::new("password", Some("salt")));

        Arc::new(Self {
            user_fixture,

            fake_remote,
            vault,
            mount_id,
            path,
            repo_id,
            cipher,
        })
    }

    pub fn new_session(&self) -> Arc<Self> {
        let user_fixture = self.user_fixture.new_session();
        let fake_remote = user_fixture.fake_remote.clone();
        let vault = user_fixture.vault.clone();

        let mount_id = self.mount_id.clone();
        let path = self.path.clone();
        let repo_id = self.repo_id.clone();
        let cipher = self.cipher.clone();

        Arc::new(Self {
            user_fixture,

            fake_remote,
            vault,
            mount_id,
            path,
            repo_id,
            cipher,
        })
    }

    pub async fn unlock(&self) {
        self.vault
            .repos_service
            .unlock_repo(&self.repo_id, "password", RepoUnlockMode::Unlock)
            .await
            .unwrap();
    }

    pub fn lock(&self) {
        self.vault.repos_service.lock_repo(&self.repo_id).unwrap();
    }

    pub async fn remove(&self) {
        self.vault
            .repos_service
            .remove_repo(&self.repo_id, "password")
            .await
            .unwrap();
    }

    pub fn encrypt_filename(&self, name: &str) -> EncryptedName {
        self.cipher.encrypt_filename(&DecryptedName(name.into()))
    }

    pub fn encrypt_path(&self, path: &str) -> EncryptedPath {
        self.cipher.encrypt_path(&DecryptedPath(path.into()))
    }

    pub fn get_file_id(&self, path: &str) -> RepoFileId {
        repo_files::selectors::get_file_id(&self.repo_id, &self.encrypt_path(path.into()))
    }

    pub async fn create_dir(&self, path: &str) -> RepoFile {
        let cipher = self.vault.repos_service.get_cipher(&self.repo_id).unwrap();
        let path = cipher.encrypt_path(&DecryptedPath(path.to_owned()));
        let (parent_path, name) = repo_encrypted_path_utils::split_parent_name(&path).unwrap();

        self.create_dir_encrypted(&parent_path, name).await
    }

    pub async fn create_dir_encrypted(
        &self,
        parent_path: &EncryptedPath,
        name: EncryptedName,
    ) -> RepoFile {
        let path = repo_encrypted_path_utils::join_path_name(parent_path, &name);

        self.vault
            .repo_files_service
            .clone()
            .create_dir_name(&self.repo_id, &parent_path, name)
            .await
            .unwrap();

        self.vault.with_state(|state| {
            state
                .repo_files
                .files
                .get(&repo_files::selectors::get_file_id(&self.repo_id, &path))
                .cloned()
                .unwrap()
        })
    }

    pub async fn upload_file(
        &self,
        path: &str,
        content: &str,
    ) -> (RepoFilesUploadResult, RepoFile) {
        let cipher = self.vault.repos_service.get_cipher(&self.repo_id).unwrap();
        let path = cipher.encrypt_path(&DecryptedPath(path.to_owned()));
        let (parent_path, name) = repo_encrypted_path_utils::split_parent_name(&path).unwrap();

        self.upload_file_encrypted(&parent_path, name, content)
            .await
    }

    pub async fn upload_file_encrypted(
        &self,
        parent_path: &EncryptedPath,
        name: EncryptedName,
        content: &str,
    ) -> (RepoFilesUploadResult, RepoFile) {
        let bytes = content.as_bytes().to_vec();
        let size = bytes.len();
        let reader = Box::pin(Cursor::new(bytes));

        let result = self
            .vault
            .repo_files_service
            .clone()
            .upload_file_reader(
                &self.repo_id,
                parent_path,
                name,
                reader,
                Some(size as i64),
                RepoFilesUploadConflictResolution::Overwrite {
                    if_remote_size: None,
                    if_remote_modified: None,
                    if_remote_hash: None,
                },
                None,
            )
            .await
            .unwrap();

        let repo_file = self.vault.with_state(|state| {
            state
                .repo_files
                .files
                .get(&result.file_id)
                .cloned()
                .unwrap()
        });

        (result, repo_file)
    }
}
