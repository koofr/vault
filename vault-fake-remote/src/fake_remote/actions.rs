use http::StatusCode;
use vault_core::{remote::models, utils::path_utils};

use super::{
    context::Context,
    errors::{ApiErrorCode, FakeRemoteError},
    files::{self, service::FilesService},
    state::{FakeRemoteState, UserContainer},
    utils::now_ms,
};

pub fn create_user(
    state: &mut FakeRemoteState,
    files_service: &FilesService,
    user_id: Option<String>,
    mount_id: Option<String>,
) -> String {
    let user_id = user_id.unwrap_or(uuid::Uuid::new_v4().to_string());

    let user: models::User = models::User {
        id: user_id.clone(),
        first_name: "Vault".into(),
        last_name: "Test".into(),
        email: user_id.replace("-", "") + "@example.com",
        phone_number: None,
        has_password: true,
        level: 1000,
    };

    let mount_id = mount_id.unwrap_or(uuid::Uuid::new_v4().to_string());

    let mount = models::Mount {
        id: mount_id.clone(),
        name: "Koofr".into(),
        typ: "device".into(),
        origin: "hosted".into(),
        online: true,
        is_primary: true,
        space_total: Some(10240),
        space_used: Some(0),
    };

    state.mounts.insert(mount_id.clone(), mount);

    let fs = files_service.create_filesystem();

    state.filesystems.insert(mount_id.clone(), fs);

    state.users.insert(
        user_id.clone(),
        UserContainer {
            user,
            mounts: vec![mount_id.clone()],
            user_vault_repos: vec![],
        },
    );

    user_id
}

pub fn create_vault_repo(
    context: &Context,
    state: &mut FakeRemoteState,
    create: models::VaultRepoCreate,
) -> Result<models::VaultRepo, FakeRemoteError> {
    let mount = state.mounts.get(&create.mount_id).ok_or_else(|| {
        FakeRemoteError::ApiError(
            StatusCode::NOT_FOUND,
            ApiErrorCode::NotFound,
            "Mount not found".into(),
        )
    })?;

    let path: files::Path = create.path.parse().map_err(|_| {
        FakeRemoteError::ApiError(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::BadRequest,
            "Invalid path".into(),
        )
    })?;

    let repo = models::VaultRepo {
        id: uuid::Uuid::new_v4().to_string(),
        name: path_utils::path_to_name(&path.0)
            .unwrap_or(&mount.name)
            .to_owned(),
        mount_id: create.mount_id,
        path: path.0,
        salt: create.salt,
        password_validator: create.password_validator,
        password_validator_encrypted: create.password_validator_encrypted,
        added: now_ms(),
    };

    state.vault_repos.insert(repo.id.clone(), repo.clone());

    state
        .users
        .get_mut(&context.user_id)
        .unwrap()
        .user_vault_repos
        .push(repo.id.clone());

    Ok(repo)
}

pub fn remove_vault_repo(
    context: &Context,
    state: &mut FakeRemoteState,
    repo_id: &str,
) -> Result<(), FakeRemoteError> {
    if state.vault_repos.remove(repo_id).is_none() {
        return Err(FakeRemoteError::ApiError(
            StatusCode::NOT_FOUND,
            ApiErrorCode::NotFound,
            "Vault repo not found".into(),
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
