use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{BodyStream, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    Form, Json,
};
use futures::TryStreamExt;
use http::{header, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeFile;
use urlencoding::encode;
use vault_core::remote::models;

use super::{
    actions,
    context::Context,
    errors::{ApiErrorCode, FakeRemoteError},
    extract::{ExtractFilesService, ExtractState},
    files,
    state::FakeRemoteState,
};

static PROFILE_PICTURE_PNG: &'static [u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x40, 0x01, 0x03, 0x00, 0x00, 0x00, 0x90, 0xA7, 0xE3,
    0x9D, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x01, 0xD9, 0xC9, 0x2C, 0x7F, 0x00, 0x00,
    0x00, 0x09, 0x70, 0x48, 0x59, 0x73, 0x00, 0x00, 0x0B, 0x13, 0x00, 0x00, 0x0B, 0x13, 0x01, 0x00,
    0x9A, 0x9C, 0x18, 0x00, 0x00, 0x00, 0x03, 0x50, 0x4C, 0x54, 0x45, 0xFF, 0xFF, 0xFF, 0xA7, 0xC4,
    0x1B, 0xC8, 0x00, 0x00, 0x00, 0x0F, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60, 0x18, 0x05,
    0xA3, 0x80, 0x7C, 0x00, 0x00, 0x02, 0x40, 0x00, 0x01, 0x59, 0x36, 0xA1, 0x03, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

pub async fn health() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct OAuth2AuthQuery {
    redirect_uri: String,
    state: String,
    user_id: Option<String>,
}

pub async fn oauth2_auth(
    ExtractState(state): ExtractState,
    Query(query): Query<OAuth2AuthQuery>,
) -> Result<Response, FakeRemoteError> {
    let mut state = state.write().unwrap();

    let user_id = query
        .user_id
        .or(state.default_user_id.clone())
        .ok_or(FakeRemoteError::BadRequest("missing user id".into()))?;

    let refresh_token = uuid::Uuid::new_v4().to_string();
    let code = uuid::Uuid::new_v4().to_string();

    state
        .oauth2_refresh_tokens
        .insert(refresh_token.clone(), user_id);
    state.oauth2_codes.insert(code.clone(), refresh_token);

    let uri = format!(
        "{}?code={}&state={}",
        query.redirect_uri,
        code,
        encode(&query.state)
    );

    Ok((
        StatusCode::SEE_OTHER,
        [(header::LOCATION, HeaderValue::try_from(uri).unwrap())],
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct OAuth2LogoutQuery {
    post_logout_redirect_uri: String,
    state: String,
}

pub async fn oauth2_logout(
    Query(query): Query<OAuth2LogoutQuery>,
) -> Result<Response, FakeRemoteError> {
    let uri = format!(
        "{}?loggedout=true&state={}",
        query.post_logout_redirect_uri,
        encode(&query.state)
    );

    Ok((
        StatusCode::SEE_OTHER,
        [(header::LOCATION, HeaderValue::try_from(uri).unwrap())],
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(tag = "grant_type")]
pub enum OAuth2TokenForm {
    #[serde(rename = "authorization_code")]
    AuthorizationCode { code: String },
    #[serde(rename = "refresh_token")]
    RefreshToken { refresh_token: String },
}

#[derive(Serialize)]
pub struct OAuth2Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
}

pub async fn oauth2_token(
    ExtractState(state): ExtractState,
    Form(form): Form<OAuth2TokenForm>,
) -> Result<Json<OAuth2Token>, FakeRemoteError> {
    let mut state = state.write().unwrap();

    let refresh_token = match form {
        OAuth2TokenForm::AuthorizationCode { code } => state
            .oauth2_codes
            .remove(&code)
            .ok_or(FakeRemoteError::Unauthorized("invalid grant".into()))?,
        OAuth2TokenForm::RefreshToken { refresh_token } => refresh_token,
    };

    let user_id = state
        .oauth2_refresh_tokens
        .get(&refresh_token)
        .ok_or(FakeRemoteError::Unauthorized("invalid grant".into()))?
        .clone();

    let access_token = uuid::Uuid::new_v4().to_string();

    state
        .oauth2_access_tokens
        .insert(access_token.clone(), user_id);

    Ok(Json(OAuth2Token {
        access_token,
        refresh_token: refresh_token.clone(),
        expires_in: 3600,
    }))
}

pub async fn user(
    ExtractState(state): ExtractState,
    context: Context,
) -> Result<Json<models::User>, FakeRemoteError> {
    let state = state.read().unwrap();
    let user = state.users.get(&context.user_id).unwrap();

    Ok(Json(user.user.clone()))
}

pub async fn content_profile_picture(_: Context) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/png")],
        PROFILE_PICTURE_PNG.to_owned(),
    )
        .into_response()
}

pub async fn user_bookmarks(_: Context) -> Result<Json<models::Bookmarks>, FakeRemoteError> {
    Ok(Json(models::Bookmarks { bookmarks: vec![] }))
}

pub async fn places(
    ExtractState(state): ExtractState,
    context: Context,
) -> Result<Json<models::Places>, FakeRemoteError> {
    let state = state.read().unwrap();

    let places = models::Places {
        places: state
            .users
            .get(&context.user_id)
            .map(|user| {
                user.mounts
                    .iter()
                    .filter_map(|mount_id| state.mounts.get(mount_id))
                    .cloned()
                    .collect()
            })
            .unwrap_or(vec![]),
    };

    Ok(Json(places))
}

pub async fn shared(_: Context) -> Result<Json<models::Shared>, FakeRemoteError> {
    Ok(Json(models::Shared { files: vec![] }))
}

fn resolve_mount_id(context: &Context, state: &FakeRemoteState, mountable: String) -> String {
    match mountable.as_str() {
        "primary" => state
            .users
            .get(&context.user_id)
            .and_then(|user| {
                user.mounts
                    .iter()
                    .find(|mount_id| {
                        state
                            .mounts
                            .get(*mount_id)
                            .filter(|mount| mount.is_primary)
                            .is_some()
                    })
                    .cloned()
            })
            .unwrap_or(mountable),
        _ => mountable,
    }
}

pub async fn mounts_details(
    ExtractState(state): ExtractState,
    context: Context,
    Path(mountable): Path<String>,
) -> Result<Json<models::Mount>, FakeRemoteError> {
    let mount = {
        let state = state.read().unwrap();

        let mount_id = resolve_mount_id(&context, &state, mountable);

        state.mounts.get(&mount_id).unwrap().clone()
    };

    Ok(Json(mount.clone()))
}

#[derive(Deserialize)]
pub struct BundleQuery {
    path: files::Path,
}

pub async fn bundle(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<BundleQuery>,
) -> Result<Json<models::Bundle>, FakeRemoteError> {
    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    let bundle = files_service.bundle(&mount_id, &query.path)?;

    Ok(Json(bundle))
}

#[derive(Deserialize)]
pub struct FilesInfoQuery {
    path: files::Path,
}

pub async fn files_info(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesInfoQuery>,
) -> Result<Json<models::FilesFile>, FakeRemoteError> {
    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    let file = files_service.info(&mount_id, &query.path)?;

    Ok(Json(file))
}

#[derive(Deserialize)]
pub struct FilesFolderNewQuery {
    path: files::Path,
}

pub async fn files_folder_new(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesFolderNewQuery>,
    Json(data): Json<models::FilesFolderCreate>,
) -> Result<StatusCode, FakeRemoteError> {
    let name: files::Name = data.name.parse().map_err(|_| {
        FakeRemoteError::ApiError(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::BadRequest,
            "Invalid name".into(),
        )
    })?;

    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    files_service
        .create_dir(&context, &mount_id, &query.path, name)
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct FilesRemoveQuery {
    path: files::Path,
    #[serde(rename = "removeIfEmpty")]
    remove_if_empty: Option<String>,
}

pub async fn files_remove(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesRemoveQuery>,
) -> Result<StatusCode, FakeRemoteError> {
    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    files_service
        .delete_file(
            &context,
            &mount_id,
            &query.path,
            query.remove_if_empty.is_some(),
        )
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct FilesRenameQuery {
    path: files::Path,
}

pub async fn files_rename(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesRenameQuery>,
    Json(data): Json<models::FilesRename>,
) -> Result<StatusCode, FakeRemoteError> {
    let name: files::Name = data.name.parse().map_err(|_| {
        FakeRemoteError::ApiError(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::BadRequest,
            "Invalid name".into(),
        )
    })?;

    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    files_service
        .rename_file(&context, &mount_id, &query.path, name)
        .await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct FilesCopyQuery {
    path: files::Path,
}

pub async fn files_copy(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesCopyQuery>,
    Json(data): Json<models::FilesCopy>,
) -> Result<Json<models::FilesCopyResult>, FakeRemoteError> {
    let (mount_id, to_mount_id) = {
        let state = state.read().unwrap();

        let mount_id = resolve_mount_id(&context, &state, mountable);

        let to_mount_id = resolve_mount_id(&context, &state, data.to_mount_id);

        (mount_id, to_mount_id)
    };

    let path = query.path;

    let to_path: files::Path = data.to_path.parse().map_err(|_| {
        FakeRemoteError::ApiError(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::BadRequest,
            "Invalid path".into(),
        )
    })?;

    if mount_id != to_mount_id {
        return Err(FakeRemoteError::BadRequest(
            "Mount id and to mount id must be the same".into(),
        ));
    }

    let to_name = to_path.name().unwrap().0;

    files_service
        .copy_file(&context, &mount_id, &path, to_path)
        .await?;

    Ok(Json(models::FilesCopyResult { name: to_name }))
}

#[derive(Deserialize)]
pub struct FilesMoveQuery {
    path: files::Path,
}

pub async fn files_move(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesMoveQuery>,
    Json(data): Json<models::FilesMove>,
) -> Result<Json<models::FilesMoveResult>, FakeRemoteError> {
    let (mount_id, to_mount_id) = {
        let state = state.read().unwrap();

        let mount_id = resolve_mount_id(&context, &state, mountable);

        let to_mount_id = resolve_mount_id(&context, &state, data.to_mount_id);

        (mount_id, to_mount_id)
    };

    let path = query.path;

    let to_path: files::Path = data.to_path.parse().map_err(|_| {
        FakeRemoteError::ApiError(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::BadRequest,
            "Invalid path".into(),
        )
    })?;

    let conditions = files::filesystem::MoveFileConditions {
        if_modified: data.if_modified,
        if_size: data.if_size,
        if_hash: data.if_hash,
    };

    if mount_id != to_mount_id {
        return Err(FakeRemoteError::BadRequest(
            "Mount id and to mount id must be the same".into(),
        ));
    }

    let to_name = to_path.name().unwrap().0;

    files_service
        .move_file(&context, &mount_id, &path, to_path, &conditions)
        .await?;

    Ok(Json(models::FilesMoveResult { name: to_name }))
}

#[derive(Deserialize)]
pub struct FilesGetQuery {
    path: files::Path,
}

pub async fn content_files_get(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesGetQuery>,
    req: http::request::Parts,
) -> Result<Response, FakeRemoteError> {
    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    let (local_path, file) = files_service.get_file_local_path(&mount_id, &query.path)?;

    let mut res = ServeFile::new_with_mime(local_path, &file.content_type.parse().unwrap())
        .try_call(http::request::Request::from_parts(req, Body::empty()))
        .await
        .map_err(|err| {
            FakeRemoteError::ApiError(
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorCode::Other,
                format!("Failed to open local file: {:?}", err),
            )
        })?;

    res.headers_mut().insert(
        HeaderName::from_lowercase(b"x-file-info").unwrap(),
        HeaderValue::from_bytes(&serde_json::to_vec(&file).unwrap()).unwrap(),
    );

    Ok(res.into_response())
}

#[derive(Deserialize)]
pub struct FilesPutQuery {
    path: files::Path,
    filename: files::Name,
    info: Option<bool>,
    autorename: Option<String>,
    overwrite: Option<String>,
    #[serde(rename = "overwriteIfModified")]
    overwrite_if_modified: Option<i64>,
    #[serde(rename = "overwriteIfSize")]
    overwrite_if_size: Option<i64>,
    #[serde(rename = "overwriteIfHash")]
    overwrite_if_hash: Option<String>,
    #[serde(rename = "overwriteIgnoreNonexisting")]
    overwrite_ignore_nonexisting: Option<String>,
    #[serde(rename = "overwriteIgnoreNonexistent")]
    overwrite_ignore_nonexistent: Option<String>,
    modified: Option<i64>,
}

pub async fn content_files_put(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesPutQuery>,
    stream: BodyStream,
) -> Result<Json<models::FilesFile>, FakeRemoteError> {
    if !matches!(query.info, Some(true)) {
        return Err(FakeRemoteError::BadRequest("Info must be true".into()));
    }

    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };
    let parent_path = query.path;
    let name = query.filename;
    let modified = query.modified;
    let conflict_resolution = files::filesystem::CreateFileConflictResolution::parse(
        query.autorename,
        query.overwrite,
        query.overwrite_if_modified,
        query.overwrite_if_size,
        query.overwrite_if_hash,
        query.overwrite_ignore_nonexisting,
        query.overwrite_ignore_nonexistent,
    );

    let reader = stream
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::BrokenPipe, err))
        .into_async_read();

    let file = files_service
        .create_file(
            &context,
            &mount_id,
            &parent_path,
            name,
            modified,
            &conflict_resolution,
            Box::pin(reader),
        )
        .await?;

    Ok(Json(file))
}

#[derive(Deserialize)]
pub struct FilesListRecursiveQuery {
    path: files::Path,
}

pub async fn content_files_list_recursive(
    ExtractState(state): ExtractState,
    ExtractFilesService(files_service): ExtractFilesService,
    context: Context,
    Path(mountable): Path<String>,
    Query(query): Query<FilesListRecursiveQuery>,
) -> Result<Response, FakeRemoteError> {
    let mount_id = {
        let state = state.read().unwrap();

        resolve_mount_id(&context, &state, mountable)
    };

    let files = files_service.list_recursive(&mount_id, &query.path)?;

    let mut buf = Vec::with_capacity(128);

    for file in files {
        serde_json::to_writer(&mut buf, &file).unwrap();

        buf.push(b'\n');
    }

    Ok((
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-ndjson; charset=utf-8"),
        )],
        buf,
    )
        .into_response())
}

pub async fn vault_repos_all(
    ExtractState(state): ExtractState,
    context: Context,
) -> Result<Json<models::VaultReposBundle>, FakeRemoteError> {
    let state = state.read().unwrap();

    let repos: Vec<_> = state
        .users
        .get(&context.user_id)
        .unwrap()
        .user_vault_repos
        .iter()
        .filter_map(|repo_id| state.vault_repos.get(repo_id))
        .cloned()
        .collect();
    let mounts: HashMap<_, _> = repos
        .iter()
        .filter_map(|repo| state.mounts.get(&repo.mount_id))
        .map(|mount| (mount.id.clone(), mount.clone()))
        .collect();

    let bundle = models::VaultReposBundle { repos, mounts };

    Ok(Json(bundle))
}

pub async fn vault_repos_create(
    ExtractState(state): ExtractState,
    context: Context,
    Json(create): Json<models::VaultRepoCreate>,
) -> Result<(StatusCode, Json<models::VaultRepo>), FakeRemoteError> {
    let mut state = state.write().unwrap();

    let create = models::VaultRepoCreate {
        mount_id: resolve_mount_id(&context, &state, create.mount_id),
        ..create
    };

    let repo = actions::create_vault_repo(&context, &mut state, create)?;

    Ok((StatusCode::CREATED, Json(repo)))
}

pub async fn vault_repos_remove(
    ExtractState(state): ExtractState,
    context: Context,
    Path(repo_id): Path<String>,
) -> Result<StatusCode, FakeRemoteError> {
    let mut state = state.write().unwrap();

    actions::remove_vault_repo(&context, &mut state, &repo_id)?;

    Ok(StatusCode::NO_CONTENT)
}
