use std::sync::{Arc, RwLock};

use http::StatusCode;
use vault_core::{
    remote::models,
    types::{MountId, RemotePath, RepoId},
    utils::path_utils,
};

use super::{
    context::Context,
    errors::{ApiErrorCode, FakeRemoteError},
    files::{self, service::FilesService, Path},
    state::FakeRemoteState,
    utils::now_ms,
};

pub struct VaultReposCreateService {
    state: Arc<RwLock<FakeRemoteState>>,
    files_service: Arc<FilesService>,
}

impl VaultReposCreateService {
    pub fn new(state: Arc<RwLock<FakeRemoteState>>, files_service: Arc<FilesService>) -> Self {
        Self {
            state,
            files_service,
        }
    }

    pub fn create_vault_repo(
        &self,
        context: &Context,
        create: models::VaultRepoCreate,
    ) -> Result<models::VaultRepo, FakeRemoteError> {
        let mount_id = create.mount_id;
        let mount_name = self
            .state
            .read()
            .unwrap()
            .mounts
            .get(&mount_id.0)
            .map(|mount| mount.name.clone())
            .ok_or_else(|| {
                FakeRemoteError::ApiError(
                    StatusCode::NOT_FOUND,
                    ApiErrorCode::NotFound,
                    "Mount not found".into(),
                    None,
                )
            })?;

        let path: Path = create.path.0.parse().map_err(|_| {
            FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::BadRequest,
                "Invalid path".into(),
                None,
            )
        })?;

        match self.files_service.info(&mount_id.0, &path) {
            Ok(_) => {}
            Err(FakeRemoteError::ApiError(_, code, _, _)) if code == ApiErrorCode::NotFound => {
                return Err(FakeRemoteError::ApiError(
                    StatusCode::NOT_FOUND,
                    ApiErrorCode::VaultReposLocationNotFound,
                    "Vault repo location not found.".into(),
                    None,
                ))
            }
            Err(err) => return Err(err),
        }

        if self
            .state
            .read()
            .unwrap()
            .vault_repos
            .values()
            .find(|repo| match Path(repo.path.0.clone()).relative_to(&path) {
                Some(path) => path.0 == "/",
                _ => false,
            })
            .is_some()
        {
            return Err(FakeRemoteError::ApiError(
                StatusCode::CONFLICT,
                ApiErrorCode::VaultReposAlreadyExists,
                "Vault repo already exists for this path.".into(),
                None,
            ));
        }

        let repo = models::VaultRepo {
            id: RepoId(uuid::Uuid::new_v4().to_string()),
            name: path_utils::path_to_name(&path.0)
                .unwrap_or(&mount_name.0)
                .to_owned(),
            mount_id,
            path: RemotePath(path.0),
            salt: create.salt,
            password_validator: create.password_validator,
            password_validator_encrypted: create.password_validator_encrypted,
            added: now_ms(),
        };

        {
            let mut state = self.state.write().unwrap();

            state.vault_repos.insert(repo.id.0.clone(), repo.clone());

            state
                .users
                .get_mut(&context.user_id)
                .unwrap()
                .user_vault_repos
                .push(repo.id.0.clone());
        }

        Ok(repo)
    }

    pub async fn create_test_vault_repo(
        &self,
        context: &Context,
    ) -> Result<models::VaultRepo, FakeRemoteError> {
        let mount_id = {
            let state = self.state.read().unwrap();

            state
                .users
                .get(&context.user_id)
                .unwrap()
                .mounts
                .first()
                .unwrap()
                .clone()
        };

        self.files_service
            .create_dir(
                &context,
                &mount_id,
                &files::Path::root(),
                files::Name("My safe box".into()),
            )
            .await?;

        self.create_vault_repo(
            &context,
            models::VaultRepoCreate {
                mount_id: MountId(mount_id),
                path: RemotePath("/My safe box".into()),
                salt: Some("salt".into()),
                password_validator: "ad3238a5-5fc7-4b8f-9575-88c69c0c91cd".into(),
                password_validator_encrypted: "v2:UkNMT05FAABVyJmka7FKh8CKL2AtIZc1xiZk-SO5GeuZPnHvw0ehM1dENa4iBCyPEf50da9V2XvL5CjpZlUle1lifEHtaRy9YHoFLHtiq1PCAqYY".into(),
            },
        )
    }
}

pub struct VaultReposRemoveService {
    state: Arc<RwLock<FakeRemoteState>>,
}

impl VaultReposRemoveService {
    pub fn new(state: Arc<RwLock<FakeRemoteState>>) -> Self {
        Self { state }
    }

    pub fn remove_vault_repo(
        &self,
        context: &Context,
        repo_id: &str,
    ) -> Result<(), FakeRemoteError> {
        let mut state = self.state.write().unwrap();

        if state.vault_repos.remove(repo_id).is_none() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::NOT_FOUND,
                ApiErrorCode::NotFound,
                "Vault repo not found".into(),
                None,
            ));
        }

        state
            .users
            .get_mut(&context.user_id)
            .unwrap()
            .user_vault_repos
            .retain(|id| id != repo_id);

        Ok(())
    }
}
