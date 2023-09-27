use std::sync::Arc;

use futures::io::Cursor;
use vault_core::{
    remote::models,
    repo_files::state::{RepoFile, RepoFilesUploadConflictResolution, RepoFilesUploadResult},
    repos::state::RepoUnlockMode,
    utils::path_utils,
    Vault,
};
use vault_fake_remote::fake_remote::{context::Context, files};

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

        let user_id = user_fixture.user_id.clone();
        let mount_id = user_fixture.mount_id.clone();
        let path = String::from("/My safe box");

        let context = Context {
            user_id,
            user_agent: None,
        };

        fake_remote
            .app_state
            .files_service
            .create_dir(
                &context,
                &mount_id,
                &files::Path::root(),
                files::Name("My safe box".into()),
            )
            .await
            .unwrap();

        let repo_id = fake_remote.app_state.vault_repos_create_service.create_vault_repo(
                &context,
                models::VaultRepoCreate {
                    mount_id: mount_id.clone(),
                    path: path.clone(),
                    salt: Some("salt".into()),
                    password_validator: "ad3238a5-5fc7-4b8f-9575-88c69c0c91cd".into(),
                    password_validator_encrypted: "v2:UkNMT05FAABVyJmka7FKh8CKL2AtIZc1xiZk-SO5GeuZPnHvw0ehM1dENa4iBCyPEf50da9V2XvL5CjpZlUle1lifEHtaRy9YHoFLHtiq1PCAqYY".into(),
                },
            )
            .unwrap().id;

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
