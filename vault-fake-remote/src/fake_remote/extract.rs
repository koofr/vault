use std::{
    convert::Infallible,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use http::{header, request::Parts, HeaderMap};

use super::{
    app_state::AppState,
    context::Context,
    errors::FakeRemoteError,
    eventstream,
    files::service::FilesService,
    state::FakeRemoteState,
    users_service::UsersService,
    vault_repos_service::{VaultReposCreateService, VaultReposRemoveService},
};

pub fn get_authorization_access_token<'a>(
    authorization: &'a str,
) -> Result<&'a str, FakeRemoteError> {
    if !authorization.starts_with("Bearer ") {
        return Err(FakeRemoteError::Unauthorized(
            "Authorization header does not start with Bearer".into(),
        ));
    }

    Ok(&authorization[7..])
}

pub fn get_headers_access_token<'a>(headers: &'a HeaderMap) -> Result<&'a str, FakeRemoteError> {
    let header_value = headers
        .get(header::AUTHORIZATION)
        .ok_or_else(|| FakeRemoteError::Unauthorized("missing Authorization header".into()))?;

    let authorization = header_value.to_str().map_err(|err| {
        FakeRemoteError::Unauthorized(format!("invalid Authorization header: {:?}", err))
    })?;

    get_authorization_access_token(authorization)
}

pub fn get_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::USER_AGENT)
        .and_then(|x| x.to_str().ok().map(|x| x.to_owned()))
}

pub fn get_user_id_by_access_token<'a>(
    state: &'a FakeRemoteState,
    access_token: &str,
) -> Result<&'a str, FakeRemoteError> {
    match state.oauth2_access_tokens.get(access_token) {
        Some(user_id) => Ok(user_id.as_str()),
        None => {
            return Err(FakeRemoteError::Unauthorized(
                "user id not found for access token".into(),
            ))
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Context {
    type Rejection = FakeRemoteError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let access_token = get_headers_access_token(&parts.headers)?;

        let state = state.state.read().unwrap();

        let user_id = get_user_id_by_access_token(&state, access_token).map(str::to_string)?;

        let user_agent = get_user_agent(&parts.headers);

        Ok(Self {
            user_id,
            user_agent,
        })
    }
}

pub struct ExtractState(pub Arc<RwLock<FakeRemoteState>>);

#[async_trait]
impl FromRequestParts<AppState> for ExtractState {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.state.clone()))
    }
}

pub struct ExtractFilesService(pub Arc<FilesService>);

#[async_trait]
impl FromRequestParts<AppState> for ExtractFilesService {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.files_service.clone()))
    }
}

pub struct ExtractUsersService(pub Arc<UsersService>);

#[async_trait]
impl FromRequestParts<AppState> for ExtractUsersService {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.users_service.clone()))
    }
}

pub struct ExtractVaultReposCreateService(pub Arc<VaultReposCreateService>);

#[async_trait]
impl FromRequestParts<AppState> for ExtractVaultReposCreateService {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.vault_repos_create_service.clone()))
    }
}

pub struct ExtractVaultReposRemoveService(pub Arc<VaultReposRemoveService>);

#[async_trait]
impl FromRequestParts<AppState> for ExtractVaultReposRemoveService {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.vault_repos_remove_service.clone()))
    }
}

pub struct ExtractEventstreamListeners(pub Arc<eventstream::Listeners>);

#[async_trait]
impl FromRequestParts<AppState> for ExtractEventstreamListeners {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.eventstream_listeners.clone()))
    }
}
