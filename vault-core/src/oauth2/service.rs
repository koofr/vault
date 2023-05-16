use std::{collections::HashMap, sync::Arc};

use data_encoding::BASE64URL_NOPAD;
use futures::lock::Mutex as AsyncMutex;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue};
use rand_core::{OsRng, RngCore};
use serde::Deserialize;
use url::Url;

use crate::{
    auth::errors::AuthError,
    common::state::Status,
    http::{HttpClient, HttpError, HttpRequest, HttpRequestBody},
    secure_storage::SecureStorageService,
    store,
};

use super::{errors::OAuth2Error, selectors, state::OAuth2Token};

const TOKEN_STORAGE_KEY: &str = "vaultOAuth2Token";
const STATE_STORAGE_KEY: &str = "vaultOAuth2State";

pub struct OAuth2Config {
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Deserialize)]
struct RawOAuth2Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
}

pub struct OAuth2Service {
    config: OAuth2Config,
    secure_storage_service: Arc<SecureStorageService>,
    http_client: Arc<Box<dyn HttpClient + Send + Sync>>,
    store: Arc<store::Store>,
    refresh_token_mutex: Arc<AsyncMutex<()>>,
}

impl OAuth2Service {
    pub fn new(
        config: OAuth2Config,
        secure_storage_service: Arc<SecureStorageService>,
        http_client: Arc<Box<dyn HttpClient + Send + Sync>>,
        store: Arc<store::Store>,
    ) -> Self {
        Self {
            config,
            secure_storage_service,
            http_client,
            store,
            refresh_token_mutex: Arc::new(AsyncMutex::new(())),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.store.with_state(selectors::select_is_authenticated)
    }

    pub fn load(&self) -> Result<(), OAuth2Error> {
        let token = self
            .secure_storage_service
            .get::<OAuth2Token>(TOKEN_STORAGE_KEY)
            .map_err(|e| OAuth2Error::InvalidOAuth2Token(e.to_string()))?;

        // TODO validate the token from storage

        self.store.mutate(store::Event::Auth, |state| {
            state.oauth2.status = match token {
                Some(_) => Status::Loaded,
                None => Status::Initial,
            };

            state.oauth2.token = token;
        });

        Ok(())
    }

    pub fn reset(&self) {
        let _ = self.secure_storage_service.remove(TOKEN_STORAGE_KEY);

        self.store.mutate(store::Event::Auth, |state| {
            state.oauth2.status = Status::Initial;
            state.oauth2.token = None;
        });
    }

    pub async fn get_authorization(&self, force_refresh_token: bool) -> Result<String, AuthError> {
        let token = match self.get_token(force_refresh_token).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                return Err(AuthError::Unauthenticated);
            }
            Err(err) => return Err(AuthError::OAuth2Error(err)),
        };

        Ok(format!("Bearer {}", token.access_token))
    }

    pub async fn get_token(
        &self,
        force_refresh_token: bool,
    ) -> Result<Option<OAuth2Token>, OAuth2Error> {
        self.refresh_token_mutex.lock().await;

        let mut token = match self.store.with_state(|state| state.oauth2.token.clone()) {
            Some(token) => token,
            None => {
                return Ok(None);
            }
        };

        if self.is_token_expired(&token) || force_refresh_token {
            token = self.refresh_token(&token.refresh_token).await?;

            self.secure_storage_service
                .set(TOKEN_STORAGE_KEY, &token)
                .unwrap();

            self.store.mutate(store::Event::Auth, |state| {
                state.oauth2.token = Some(token.clone());
            });
        }

        Ok(Some(token))
    }

    pub fn start_flow(&self) -> String {
        let flow_state = self.generate_flow_state();

        let auth_url = self.get_auth_url(&flow_state);

        self.secure_storage_service
            .set(STATE_STORAGE_KEY, &flow_state)
            .unwrap();

        auth_url
    }

    pub async fn finish_flow_url(&self, url: &str) -> Result<(), OAuth2Error> {
        let (code, state) = match self.parse_url(url) {
            Ok(x) => x,
            Err(err) => {
                self.store.mutate(store::Event::Auth, |state| {
                    state.oauth2.status = Status::Error { error: err.clone() };
                });

                return Err(err);
            }
        };

        self.finish_flow(&code, &state).await
    }

    fn parse_url(&self, url: &str) -> Result<(String, String), OAuth2Error> {
        let parsed_url = Url::parse(url)
            .map_err(|e| OAuth2Error::Unknown(format!("invalid url: {}", e.to_string())))?;
        let query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

        if let Some(error_description) = query.get("error_description") {
            return Err(OAuth2Error::Unknown(error_description.to_owned()));
        }

        if let Some(error) = query.get("error") {
            return Err(OAuth2Error::Unknown(error.to_owned()));
        }

        let code = query.get("code").ok_or(OAuth2Error::Unknown(format!(
            "code missing in url: {}",
            url
        )))?;

        let state = query.get("state").ok_or(OAuth2Error::Unknown(format!(
            "state missing in url: {}",
            url
        )))?;

        Ok((code.to_owned(), state.to_owned()))
    }

    pub async fn finish_flow(&self, code: &str, state: &str) -> Result<(), OAuth2Error> {
        self.store.mutate(store::Event::Auth, |state| {
            state.oauth2.status = Status::Loading;
        });

        if !self.is_state_ok(state) {
            self.store.mutate(store::Event::Auth, |state| {
                state.oauth2.status = Status::Error {
                    error: OAuth2Error::InvalidOAuth2State,
                };
            });

            return Err(OAuth2Error::InvalidOAuth2State);
        }

        let token = match self
            .exchange_token("authorization_code", "code", code)
            .await
        {
            Ok(token) => token,
            Err(err) => {
                self.store.mutate(store::Event::Auth, |state| {
                    state.oauth2.status = Status::Error { error: err.clone() };
                });

                return Err(err);
            }
        };

        self.secure_storage_service
            .set(TOKEN_STORAGE_KEY, &token)
            .unwrap();

        self.store.mutate(store::Event::Auth, |state| {
            state.oauth2.status = Status::Loaded;
            state.oauth2.token = Some(token);
        });

        let _ = self.secure_storage_service.remove(STATE_STORAGE_KEY);

        Ok(())
    }

    fn is_state_ok(&self, state: &str) -> bool {
        match self.secure_storage_service.get::<String>(STATE_STORAGE_KEY) {
            Ok(Some(saved_state)) => state == saved_state,
            _ => false,
        }
    }

    fn get_auth_url(&self, state: &str) -> String {
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert("state", state);
        params.insert("response_type", "code");
        params.insert("scope", "public");

        let mut auth_url = Url::parse(&format!("{}/oauth2/auth", &self.config.base_url)).unwrap();
        auth_url.query_pairs_mut().extend_pairs(&params);

        auth_url.to_string()
    }

    fn get_token_url(&self) -> String {
        format!("{}/oauth2/token", &self.config.base_url)
    }

    fn generate_flow_state(&self) -> String {
        let mut state = vec![0; 16];

        (&mut OsRng).try_fill_bytes(&mut state).unwrap();

        BASE64URL_NOPAD.encode(&state)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuth2Token, OAuth2Error> {
        self.exchange_token("refresh_token", "refresh_token", refresh_token)
            .await
    }

    fn is_token_expired(&self, token: &OAuth2Token) -> bool {
        // 10 minutes for clock skew
        token.expires_at < instant::now() - 10.0 * 60.0
    }

    async fn exchange_token(
        &self,
        grant_type: &str,
        key: &str,
        value: &str,
    ) -> Result<OAuth2Token, OAuth2Error> {
        let mut params = HashMap::new();
        params.insert("grant_type", grant_type);
        params.insert("client_id", &self.config.client_id);
        params.insert("client_secret", &self.config.client_secret);
        params.insert("redirect_uri", &self.config.redirect_uri);
        params.insert(key, value);

        let body = serde_urlencoded::to_string(params).unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let res = self
            .http_client
            .request(HttpRequest {
                method: String::from("POST"),
                url: self.get_token_url(),
                headers,
                body: Some(HttpRequestBody::Bytes(body.into_bytes())),
                ..Default::default()
            })
            .await?;

        let status_code = res.status_code();

        if status_code != 200 {
            let bytes = res.bytes().await.map_err(OAuth2Error::HttpError)?;
            let str = String::from_utf8(bytes).unwrap_or(String::from("non-utf8 response"));

            if status_code == 401 {
                return Err(OAuth2Error::InvalidGrant(str));
            }

            return Err(OAuth2Error::HttpError(HttpError::ResponseError(format!(
                "unexpected status: {}: {}",
                status_code, &str,
            ))));
        }

        let res_bytes = res.bytes().await?;

        let raw_token: RawOAuth2Token = serde_json::from_slice(&res_bytes)
            .map_err(|e| OAuth2Error::InvalidOAuth2Token(e.to_string()))?;

        let token = OAuth2Token {
            access_token: raw_token.access_token,
            refresh_token: raw_token.refresh_token,
            expires_at: instant::now() + raw_token.expires_in as f64 * 1000.0,
        };

        Ok(token)
    }
}
