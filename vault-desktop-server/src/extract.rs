use std::{convert::Infallible, sync::Arc};

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

use vault_web_api::web_vault_base::WebVaultBase;

use crate::{app_state::AppState, callbacks::Callbacks, request_id::RequestId, sessions::Sessions};

pub struct ExtractBase(pub Arc<WebVaultBase>);

impl FromRequestParts<AppState> for ExtractBase {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.base.clone()))
    }
}

pub struct ExtractSessions(pub Arc<Sessions>);

impl FromRequestParts<AppState> for ExtractSessions {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        Ok(Self(state.sessions.clone()))
    }
}

pub struct ExtractCallbacks(pub Arc<Callbacks>);

impl FromRequestParts<AppState> for ExtractCallbacks {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let request_id = parts
            .extensions
            .get::<RequestId>()
            .ok_or_else(|| (StatusCode::FORBIDDEN, "request without session").into_response())?;
        let callbacks = state
            .sessions
            .get_callbacks(request_id)
            .map_err(|err| (StatusCode::FORBIDDEN, err.to_string()).into_response())?;

        Ok(Self(callbacks))
    }
}
