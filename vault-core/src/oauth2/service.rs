use std::{collections::HashMap, sync::Arc};

use data_encoding::BASE64URL_NOPAD;
use futures::lock::Mutex as AsyncMutex;
use http::{header::CONTENT_TYPE, HeaderMap, HeaderValue};
use rand_core::{OsRng, RngCore};
use serde::Deserialize;
use url::Url;

use crate::{
    auth::errors::AuthError,
    http::{HttpClient, HttpError, HttpRequest, HttpRequestBody},
    runtime,
    secure_storage::{errors::SecureStorageError, SecureStorageService},
    store,
};

use super::{
    errors::OAuth2Error,
    mutations, selectors,
    state::{FinishFlowResult, OAuth2Token},
};

pub const TOKEN_STORAGE_KEY: &str = "vaultOAuth2Token";
pub const STATE_STORAGE_KEY: &str = "vaultOAuth2State";

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
    runtime: Arc<runtime::BoxRuntime>,

    refresh_token_mutex: Arc<AsyncMutex<()>>,
}

impl OAuth2Service {
    pub fn new(
        config: OAuth2Config,
        secure_storage_service: Arc<SecureStorageService>,
        http_client: Arc<Box<dyn HttpClient + Send + Sync>>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Self {
        Self {
            config,
            secure_storage_service,
            http_client,
            store,
            runtime,

            refresh_token_mutex: Arc::new(AsyncMutex::new(())),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.store.with_state(selectors::select_is_authenticated)
    }

    pub fn load(&self) -> Result<(), OAuth2Error> {
        let res = self.load_token().map_err(OAuth2Error::from);
        let res_err = res.as_ref().map(|_| ()).map_err(|err| err.clone());

        self.store.mutate(|state, notify, _, _| {
            mutations::loaded(state, notify, res);
        });

        res_err
    }

    pub fn logout(&self) -> Result<(), OAuth2Error> {
        let res = self.remove_token().map_err(OAuth2Error::StorageError);

        self.store.mutate(|state, notify, _, _| {
            mutations::logout(state, notify, res.clone());
        });

        res
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
        let refresh_token_guard = self.refresh_token_mutex.lock().await;

        let mut token = match self.store.with_state(|state| state.oauth2.token.clone()) {
            Some(token) => token,
            None => {
                return Ok(None);
            }
        };

        if self.is_token_expired(&token) || force_refresh_token {
            token = self.refresh_token(&token.refresh_token).await?;

            self.save_token(&token)?;

            self.store.mutate(|state, notify, _, _| {
                mutations::update_token(state, notify, token.clone());
            });
        }

        drop(refresh_token_guard);

        Ok(Some(token))
    }

    pub fn start_login_flow(&self) -> Result<String, OAuth2Error> {
        let flow_state = self.generate_flow_state()?;

        Ok(self.get_login_url(&flow_state))
    }

    pub fn start_logout_flow(&self) -> Result<String, OAuth2Error> {
        let flow_state = self.generate_flow_state()?;

        Ok(self.get_logout_url(&flow_state))
    }

    pub async fn finish_flow_url(&self, url: &str) -> Result<FinishFlowResult, OAuth2Error> {
        let query = match self.parse_url(url) {
            Ok(query) => query,
            Err(err) => {
                self.handle_error(err.clone());

                return Err(err);
            }
        };

        let state = match query.get("state").ok_or(OAuth2Error::Unknown(format!(
            "state missing in url: {}",
            url
        ))) {
            Ok(state) => state,
            Err(err) => {
                self.handle_error(err.clone());

                return Err(err);
            }
        };

        if !self.is_flow_state_ok(state) {
            let err = OAuth2Error::InvalidOAuth2State;

            self.handle_error(err.clone());

            return Err(err);
        }

        if query.get("loggedout").filter(|&x| x == "true").is_some() {
            self.finish_logout_flow()?;

            Ok(FinishFlowResult::LoggedOut)
        } else {
            let code = match query.get("code").ok_or(OAuth2Error::Unknown(format!(
                "code missing in url: {}",
                url
            ))) {
                Ok(state) => state,
                Err(err) => {
                    self.handle_error(err.clone());

                    return Err(err);
                }
            };

            self.finish_login_flow(&code).await?;

            Ok(FinishFlowResult::LoggedIn)
        }
    }

    fn parse_url(&self, url: &str) -> Result<HashMap<String, String>, OAuth2Error> {
        let parsed_url = Url::parse(url)
            .map_err(|e| OAuth2Error::Unknown(format!("invalid url: {}", e.to_string())))?;
        let query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

        if let Some(error_description) = query.get("error_description") {
            return Err(OAuth2Error::Unknown(error_description.to_owned()));
        }

        if let Some(error) = query.get("error") {
            return Err(OAuth2Error::Unknown(error.to_owned()));
        }

        Ok(query)
    }

    fn handle_error(&self, err: OAuth2Error) {
        self.store.mutate(|state, notify, _, _| {
            mutations::error(state, notify, err);
        });
    }

    async fn finish_login_flow(&self, code: &str) -> Result<(), OAuth2Error> {
        self.store.mutate(|state, notify, _, _| {
            mutations::logging_in(state, notify);
        });

        let token = match self
            .exchange_token("authorization_code", "code", code)
            .await
        {
            Ok(token) => token,
            Err(err) => {
                let _ = self.remove_state();

                self.handle_error(err.clone());

                return Err(err);
            }
        };

        match self.save_token(&token) {
            Ok(()) => {}
            Err(err) => {
                let _ = self.remove_state();

                self.handle_error(err.clone().into());

                return Err(err.into());
            }
        }

        if let Err(err) = self.remove_state().map_err(OAuth2Error::from) {
            self.handle_error(err.clone());

            return Err(err);
        }

        self.store.mutate(|state, notify, _, _| {
            mutations::logged_in(state, notify, token);
        });

        Ok(())
    }

    fn finish_logout_flow(&self) -> Result<(), OAuth2Error> {
        if let Err(err) = self.remove_state().map_err(OAuth2Error::from) {
            self.handle_error(err.clone());

            return Err(err);
        }

        self.logout()
    }

    fn is_flow_state_ok(&self, state: &str) -> bool {
        match self.load_state() {
            Ok(Some(saved_state)) => state == saved_state,
            _ => false,
        }
    }

    fn get_login_url(&self, state: &str) -> String {
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

    fn get_logout_url(&self, state: &str) -> String {
        let mut params: HashMap<&str, &str> = HashMap::new();
        params.insert("client_id", &self.config.client_id);
        params.insert("post_logout_redirect_uri", &self.config.redirect_uri);
        params.insert("state", state);

        let mut logout_url =
            Url::parse(&format!("{}/oauth2/logout", &self.config.base_url)).unwrap();
        logout_url.query_pairs_mut().extend_pairs(&params);

        logout_url.to_string()
    }

    fn get_token_url(&self) -> String {
        format!("{}/oauth2/token", &self.config.base_url)
    }

    fn generate_flow_state(&self) -> Result<String, OAuth2Error> {
        let mut state = vec![0; 16];

        (&mut OsRng).try_fill_bytes(&mut state).unwrap();

        let state = BASE64URL_NOPAD.encode(&state);

        self.save_state(&state)?;

        Ok(state)
    }

    async fn refresh_token(&self, refresh_token: &str) -> Result<OAuth2Token, OAuth2Error> {
        self.exchange_token("refresh_token", "refresh_token", refresh_token)
            .await
    }

    fn is_token_expired(&self, token: &OAuth2Token) -> bool {
        // 10 minutes for clock skew
        (token.expires_at as i64) < self.runtime.now_ms() - 10 * 60 * 1000
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
            .map_err(|err| OAuth2Error::InvalidOAuth2Token(err.to_string()))?;

        let token = OAuth2Token {
            access_token: raw_token.access_token,
            refresh_token: raw_token.refresh_token,
            expires_at: (self.runtime.now_ms() as f64) + (raw_token.expires_in as f64) * 1000.0,
        };

        Ok(token)
    }

    fn load_token(&self) -> Result<Option<OAuth2Token>, SecureStorageError> {
        self.secure_storage_service.get(TOKEN_STORAGE_KEY)
    }

    fn save_token(&self, token: &OAuth2Token) -> Result<(), SecureStorageError> {
        self.secure_storage_service.set(TOKEN_STORAGE_KEY, &token)
    }

    fn remove_token(&self) -> Result<(), SecureStorageError> {
        self.secure_storage_service.remove(TOKEN_STORAGE_KEY)
    }

    fn load_state(&self) -> Result<Option<String>, SecureStorageError> {
        self.secure_storage_service.get(STATE_STORAGE_KEY)
    }

    fn save_state(&self, state: &str) -> Result<(), SecureStorageError> {
        self.secure_storage_service.set(STATE_STORAGE_KEY, &state)
    }

    fn remove_state(&self) -> Result<(), SecureStorageError> {
        self.secure_storage_service.remove(STATE_STORAGE_KEY)
    }
}
