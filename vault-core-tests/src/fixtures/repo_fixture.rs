use std::sync::Arc;

use futures::io::Cursor;
use vault_core::{
    repo_files::state::{RepoFile, RepoFilesUploadConflictResolution, RepoFilesUploadResult},
    repos::state::RepoUnlockMode,
    utils::path_utils,
    Vault,
};
use vault_fake_remote::fake_remote::context::Context;

use crate::{fake_remote::FakeRemote, fixtures::user_fixture::UserFixture};

pub struct RepoFixture {
    pub user_fixture: Arc<UserFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub vault: Arc<Vault>,
    pub mount_id: String,
    pub path: String,
    pub repo_id: String,
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

        let repo_id = repo.id;
        let mount_id = repo.mount_id;
        let path = repo.path;

        Arc::new(Self {
            user_fixture,

            fake_remote,
            vault,
            mount_id,
            path,
            repo_id,
        })
    }

    pub async fn unlock(&self) {
        self.vault
            .repos_service
            .unlock_repo(&self.repo_id, "password", RepoUnlockMode::Unlock)
            .await
            .unwrap();
    }

    pub async fn create_dir(&self, path: &str) {
        let (parent_path, name) = path_utils::split_parent_name(path).unwrap();

        self.vault
            .repo_files_service
            .clone()
            .create_dir_name(&self.repo_id, parent_path, name)
            .await
            .unwrap();
    }

    pub async fn upload_file(
        &self,
        path: &str,
        content: &str,
    ) -> (RepoFilesUploadResult, RepoFile) {
        let (parent_path, name) = path_utils::split_parent_name(path).unwrap();

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
                RepoFilesUploadConflictResolution::Error,
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
