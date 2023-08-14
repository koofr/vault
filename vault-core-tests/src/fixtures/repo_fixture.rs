use std::sync::Arc;

use futures::io::Cursor;
use vault_core::{
    remote::models, repo_files::state::RepoFilesUploadConflictResolution,
    repos::state::RepoUnlockMode, utils::path_utils,
};
use vault_fake_remote::fake_remote::{actions, context::Context, files};

use crate::fixtures::user_fixture::UserFixture;

pub struct RepoFixture {
    pub user_fixture: Arc<UserFixture>,

    pub mount_id: String,
    pub path: String,
    pub repo_id: String,
}

impl RepoFixture {
    pub async fn create(user_fixture: Arc<UserFixture>) -> Self {
        let fake_remote = user_fixture.fake_remote.clone();

        let user_id = user_fixture.user_id.clone();
        let mount_id = user_fixture.mount_id.clone();
        let path = String::from("/My safe box");

        let context = Context {
            user_id,
            user_agent: None,
        };

        fake_remote
            .files_service
            .create_dir(
                &context,
                &mount_id,
                &files::Path::root(),
                files::Name("My safe box".into()),
            )
            .await
            .unwrap();

        let repo_id = {
            let mut state = fake_remote.state.write().unwrap();

            actions::create_vault_repo(
                &context,
                &mut state,
                models::VaultRepoCreate {
                    mount_id: mount_id.clone(),
                    path: path.clone(),
                    salt: Some("salt".into()),
                    password_validator: "ad3238a5-5fc7-4b8f-9575-88c69c0c91cd".into(),
                    password_validator_encrypted: "v2:UkNMT05FAABVyJmka7FKh8CKL2AtIZc1xiZk-SO5GeuZPnHvw0ehM1dENa4iBCyPEf50da9V2XvL5CjpZlUle1lifEHtaRy9YHoFLHtiq1PCAqYY".into(),
                },
            )
            .unwrap().id
        };

        Self {
            user_fixture,

            mount_id,
            path,
            repo_id,
        }
    }

    pub async fn unlock(&self) {
        self.user_fixture
            .vault
            .repos_service
            .unlock_repo(&self.repo_id, "password", RepoUnlockMode::Unlock)
            .await
            .unwrap();
    }

    pub async fn upload_file(&self, path: &str, content: &str) {
        let (parent_path, name) = path_utils::split_parent_name(path).unwrap();

        let bytes = content.as_bytes().to_vec();
        let size = bytes.len();
        let reader = Box::pin(Cursor::new(bytes));

        self.user_fixture
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
    }
}