use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use futures::{
    stream::{BoxStream, TryStreamExt},
    AsyncBufReadExt, StreamExt,
};
use http::{header, HeaderMap, HeaderValue};
use serde::Serialize;
use urlencoding::encode;

use crate::{
    auth,
    auth::errors::AuthError,
    common::state::BoxAsyncRead,
    http::{BoxHttpResponse, HttpClient, HttpError, HttpRequest, HttpRequestBody},
    oauth2::errors::OAuth2Error,
    types::{MountId, RemoteName, RemotePath, RepoId},
};

use super::{
    errors::RemoteError,
    models::{self, ApiError},
    ApiErrorCode,
};

pub type ListRecursiveItemStream =
    BoxStream<'static, Result<models::FilesListRecursiveItem, RemoteError>>;

pub struct RemoteFileReader {
    pub file: models::FilesFile,
    pub size: i64,
    pub reader: BoxAsyncRead,
}

#[derive(Debug, Clone)]
pub enum RemoteFileUploadConflictResolution {
    Autorename,
    Overwrite {
        if_size: Option<i64>,
        if_modified: Option<i64>,
        if_hash: Option<String>,
        ignore_nonexisting: bool,
    },
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFileRemoveConditions {
    pub if_empty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFileMoveConditions {
    pub if_size: Option<i64>,
    pub if_modified: Option<i64>,
    pub if_hash: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RemoteFileTagsSetConditions {
    pub if_size: Option<i64>,
    pub if_modified: Option<i64>,
    pub if_hash: Option<String>,
    pub if_old_tags: Option<HashMap<String, Vec<String>>>,
}

pub type Logout = Box<dyn Fn() + Send + Sync + 'static>;

pub struct Remote {
    base_url: String,
    http_client: Arc<Box<dyn HttpClient + Send + Sync>>,
    auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,

    logout: Arc<RwLock<Option<Logout>>>,
    user_agent: Option<String>,
}

impl Remote {
    pub fn new(
        base_url: String,
        http_client: Arc<Box<dyn HttpClient + Send + Sync>>,
        auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,
    ) -> Remote {
        Remote {
            base_url,
            http_client,
            auth_provider,

            logout: Arc::new(RwLock::new(None)),
            user_agent: None,
        }
    }

    pub fn set_logout(&self, logout: Logout) {
        let mut logout_guard = self.logout.write().unwrap();

        *logout_guard = Some(logout)
    }

    pub fn with_useragent(&self, user_agent: Option<String>) -> Self {
        Self {
            base_url: self.base_url.clone(),
            http_client: self.http_client.clone(),
            auth_provider: self.auth_provider.clone(),

            logout: self.logout.clone(),
            user_agent,
        }
    }

    async fn request(&self, mut request: HttpRequest) -> Result<BoxHttpResponse, RemoteError> {
        request.set_base_url(&self.base_url);

        if let Some(user_agent) = &self.user_agent {
            request.headers.insert(
                header::USER_AGENT,
                HeaderValue::from_str(user_agent).unwrap(),
            );
        }

        let is_req_retriable = request.is_retriable;

        let request_clone = match request.try_clone() {
            Some(request_clone) => request_clone,
            None => return self.request_inner(request).await,
        };

        match self.request_inner(request_clone).await {
            Ok(res) => {
                if is_res_unauthorized(&res) || (is_req_retriable && is_res_server_error(&res)) {
                    match self.request_inner(request).await {
                        Ok(retried_res) => {
                            if is_res_server_error(&retried_res) {
                                // on retried response server error return the original response
                                Ok(res)
                            } else {
                                Ok(retried_res)
                            }
                        }
                        // on retry error return the original response
                        Err(_) => Ok(res),
                    }
                } else {
                    Ok(res)
                }
            }
            Err(err) if is_req_retriable => {
                match self.request_inner(request).await {
                    Ok(res) => Ok(res),
                    // on retry error return the original error
                    Err(_) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    async fn request_inner(
        &self,
        mut request: HttpRequest,
    ) -> Result<BoxHttpResponse, RemoteError> {
        let authorization = self.get_authorization(false).await?;

        request
            .headers
            .insert(header::AUTHORIZATION, authorization.parse().unwrap());

        let res = self
            .http_client
            .request(request)
            .await
            .map_err(RemoteError::HttpError)?;

        // invalid oauth2 token
        if is_res_unauthorized(&res) {
            // try to refresh the token
            let _ = self.get_authorization(true).await;
        }

        return Ok(res);
    }

    pub async fn get_authorization(
        &self,
        force_refresh_token: bool,
    ) -> Result<String, RemoteError> {
        let res = self
            .auth_provider
            .get_authorization(force_refresh_token)
            .await;

        match res {
            Err(AuthError::OAuth2Error(OAuth2Error::InvalidGrant(_))) => {
                if let Some(logout) = &*self.logout.read().unwrap() {
                    logout();
                }
            }
            _ => {}
        }

        Ok(res.map_err(|e| HttpError::ResponseError(e.to_string()))?)
    }

    pub async fn get_user(&self) -> Result<models::User, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!("/api/v2.1/user"),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn get_profile_picture_bytes(&self, user_id: &str) -> Result<Vec<u8>, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/content/api/v2.1/users/{}/profile-picture?nodefault",
                    user_id,
                ),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res.bytes().await.map_err(RemoteError::HttpError)
    }

    pub async fn get_mount(&self, id: &MountId) -> Result<models::Mount, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!("/api/v2.1/mounts/{}", &id.0),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn get_vault_repos(&self) -> Result<models::VaultReposBundle, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!("/api/v2.1/vault/repos"),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn create_vault_repo(
        &self,
        create: models::VaultRepoCreate,
    ) -> Result<models::VaultRepo, RemoteError> {
        let (req_body, req_headers) = req_json(&create);

        let res = self
            .request(HttpRequest {
                method: String::from("POST"),
                url: format!("/api/v2.1/vault/repos"),
                headers: req_headers,
                body: req_body,
                is_retriable: false,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 201 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn remove_vault_repo(&self, repo_id: &RepoId) -> Result<(), RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("DELETE"),
                url: format!("/api/v2.1/vault/repos/{}", repo_id.0),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 204 {
            return res_error(res).await;
        }

        Ok(())
    }

    pub async fn get_places(&self) -> Result<Vec<models::Mount>, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: String::from("/api/v2.1/places"),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        let places: models::Places = res_json(res).await?;

        Ok(places.places)
    }

    pub async fn get_bookmarks(&self) -> Result<Vec<models::Bookmark>, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: String::from("/api/v2.1/user/bookmarks"),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        let bookmarks: models::Bookmarks = res_json(res).await?;

        Ok(bookmarks.bookmarks)
    }

    pub async fn get_shared(&self) -> Result<Vec<models::SharedFile>, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: String::from("/api/v2.1/shared"),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        let shared: models::Shared = res_json(res).await?;

        Ok(shared.files)
    }

    pub async fn get_bundle(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<models::Bundle, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/api/v2.1/mounts/{}/bundle?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn get_list_recursive(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<ListRecursiveItemStream, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/content/api/v2.1/mounts/{}/files/listrecursive?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        let reader = res
            .bytes_stream()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            .into_async_read();

        let lines_stream = reader.lines();

        let items_stream = lines_stream.map(|item| match item {
            Ok(line) => serde_json::from_str(&line).map_err(|e| {
                RemoteError::HttpError(HttpError::ResponseError(format!(
                    "json deserialize error: {}",
                    e.to_string()
                )))
            }),
            Err(err) => Err(RemoteError::HttpError(
                err.into_inner().unwrap().downcast_ref().cloned().unwrap(),
            )),
        });

        Ok(Box::pin(items_stream))
    }

    pub async fn get_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<models::FilesFile, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/info?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn get_file_reader(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
    ) -> Result<RemoteFileReader, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/content/api/v2.1/mounts/{}/files/get?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        let file_info_header = res.headers().get("X-File-Info").ok_or_else(|| {
            RemoteError::HttpError(HttpError::ResponseError(String::from(
                "Missing response header X-File-Info",
            )))
        })?;
        let file: models::FilesFile =
            serde_json::from_slice(file_info_header.as_bytes()).map_err(|e| {
                RemoteError::HttpError(HttpError::ResponseError(format!(
                    "file info json deserialize error: {}",
                    e.to_string()
                )))
            })?;

        let size = res
            .headers()
            .get("Content-Length")
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();

        let reader = res
            .bytes_stream()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            .into_async_read();

        Ok(RemoteFileReader {
            file,
            size,
            reader: Box::pin(reader),
        })
    }

    pub async fn upload_file_reader(
        &self,
        mount_id: &MountId,
        parent_path: &RemotePath,
        name: &RemoteName,
        reader: BoxAsyncRead,
        size: Option<i64>,
        modified: Option<i64>,
        conflict_resolution: RemoteFileUploadConflictResolution,
        on_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
    ) -> Result<models::FilesFile, RemoteError> {
        let (
            autorename,
            overwrite,
            overwrite_if_size,
            overwrite_if_modified,
            overwrite_if_hash,
            overwrite_ignore_nonexisting,
        ) = match conflict_resolution {
            RemoteFileUploadConflictResolution::Autorename => {
                (true, false, None, None, None, false)
            }
            RemoteFileUploadConflictResolution::Overwrite {
                if_size,
                if_modified,
                if_hash,
                ignore_nonexisting,
            } => (
                false,
                true,
                if_size,
                if_modified,
                if_hash,
                ignore_nonexisting,
            ),
            RemoteFileUploadConflictResolution::Error => (false, false, None, None, None, false),
        };

        let mut url = format!(
            "/content/api/v2.1/mounts/{}/files/put?path={}&filename={}&autorename={}&overwrite={}&info=true",
            &mount_id.0,
            encode(&parent_path.0),
            encode(&name.0),
            autorename,
            overwrite,
        );

        if let Some(size) = size {
            url = format!("{}&size={}", url, size);
        }
        if let Some(modified) = modified {
            url = format!("{}&modified={}", url, modified);
        }
        if let Some(overwrite_if_size) = overwrite_if_size {
            url = format!("{}&overwriteIfSize={}", url, overwrite_if_size);
        }
        if let Some(overwrite_if_modified) = overwrite_if_modified {
            url = format!("{}&overwriteIfModified={}", url, overwrite_if_modified);
        }
        if let Some(overwrite_if_hash) = overwrite_if_hash {
            url = format!("{}&overwriteIfHash={}", url, overwrite_if_hash);
        }
        if overwrite_ignore_nonexisting {
            url = format!("{}&overwriteIgnoreNonexisting=", url);
        }

        let res = self
            .request(HttpRequest {
                method: String::from("POST"),
                url,
                headers: HeaderMap::new(),
                body: Some(HttpRequestBody::Reader(reader)),
                on_body_progress: on_progress,
                is_retriable: false,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn delete_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        conditions: RemoteFileRemoveConditions,
    ) -> Result<(), RemoteError> {
        let mut url = format!(
            "/api/v2.1/mounts/{}/files/remove?path={}",
            &mount_id.0,
            encode(&path.0)
        );

        if conditions.if_empty {
            url = format!("{}&removeIfEmpty=", url);
        }

        let res = self
            .request(HttpRequest {
                method: String::from("DELETE"),
                url,
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        Ok(())
    }

    pub async fn create_dir(
        &self,
        mount_id: &MountId,
        parent_path: &RemotePath,
        name: RemoteName,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesFolderCreate { name });

        let res = self
            .request(HttpRequest {
                method: String::from("POST"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/folder?path={}",
                    &mount_id.0,
                    encode(&parent_path.0)
                ),
                headers: req_headers,
                body: req_body,
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        Ok(())
    }

    pub async fn rename_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        new_name: RemoteName,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesRename { name: new_name });

        let res = self
            .request(HttpRequest {
                method: String::from("PUT"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/rename?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                headers: req_headers,
                body: req_body,
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        Ok(())
    }

    pub async fn copy_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        to_mount_id: &MountId,
        to_path: &RemotePath,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesCopy {
            to_mount_id: to_mount_id.to_owned(),
            to_path: to_path.to_owned(),
        });

        let res = self
            .request(HttpRequest {
                method: String::from("PUT"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/copy?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                headers: req_headers,
                body: req_body,
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        Ok(())
    }

    pub async fn move_file(
        &self,
        mount_id: &MountId,
        path: &RemotePath,
        to_mount_id: &MountId,
        to_path: &RemotePath,
        conditions: RemoteFileMoveConditions,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesMove {
            to_mount_id: to_mount_id.to_owned(),
            to_path: to_path.to_owned(),
            if_modified: conditions.if_modified,
            if_size: conditions.if_size,
            if_hash: conditions.if_hash,
        });

        let res = self
            .request(HttpRequest {
                method: String::from("PUT"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/move?path={}",
                    &mount_id.0,
                    encode(&path.0)
                ),
                headers: req_headers,
                body: req_body,
                is_retriable: true,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        Ok(())
    }
}

pub fn req_json<T>(value: &T) -> (Option<HttpRequestBody>, HeaderMap)
where
    T: ?Sized + Serialize,
{
    let body = serde_json::to_vec(value).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    (Some(HttpRequestBody::Bytes(body)), headers)
}

async fn res_bytes(res: BoxHttpResponse) -> Result<Vec<u8>, RemoteError> {
    res.bytes().await.map_err(RemoteError::HttpError)
}

async fn res_error<T>(res: BoxHttpResponse) -> Result<T, RemoteError> {
    let status_code = res.status_code();

    let is_content_type_json = res
        .headers()
        .get(header::CONTENT_TYPE)
        .filter(|x| x.as_bytes() == b"application/json; charset=utf-8")
        .is_some();

    let bytes = res_bytes(res).await?;

    if is_content_type_json {
        match serde_json::from_slice::<ApiError>(&bytes) {
            Ok(api_error) => return Err(RemoteError::from_api_error(api_error, status_code)),
            _ => (),
        }
    }

    match status_code {
        404 => {
            return Err(RemoteError::from_code(ApiErrorCode::NotFound, "Not found"));
        }
        _ => (),
    }

    let message = String::from_utf8(bytes).unwrap_or(String::from("non-utf8 response"));

    Err(RemoteError::UnexpectedStatus {
        status_code,
        message,
    })
}

async fn res_json<'a, T>(res: BoxHttpResponse) -> Result<T, RemoteError>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = res_bytes(res).await?;

    serde_json::from_slice(&bytes).map_err(|e| {
        RemoteError::HttpError(HttpError::ResponseError(format!(
            "json deserialize error: {}",
            e.to_string()
        )))
    })
}

fn is_res_unauthorized(res: &BoxHttpResponse) -> bool {
    res.status_code() == 401
}

fn is_res_server_error(res: &BoxHttpResponse) -> bool {
    res.status_code() >= 500
}

#[cfg(test)]
pub mod tests {
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };

    use futures::{executor::block_on, stream, StreamExt};
    use http::HeaderMap;

    use crate::{
        auth::{errors::AuthError, mock_auth_provider::MockAuthProvider, AuthProvider},
        http::{
            mock_http_client::{MockHttpClient, MockHttpResponse},
            HttpClient, HttpError, HttpRequest,
        },
        oauth2::errors::OAuth2Error,
        remote::{models, RemoteError},
        types::{MountId, RemoteName, RemotePath},
    };

    use super::Remote;

    const USER_JSON: &'static str = r#"{"id":"30bce243-bae7-40d3-9f6f-782fb060c3e7","firstName":"Test","lastName":"User","email":"test@example.com","level":1000,"hasPassword":true}"#;
    const VAULT_REPO_JSON: &'static str = r#"{"id":"e85595ea-3779-4e8b-515e-f75ef67c5004","name":"My safe box","mountId":"aedbf61c-b0e6-4717-457e-dc592e303e3e","path":"/My safe box","salt":"salt","passwordValidator":"d743454e-894e-41ca-ac11-dab6245bca99","passwordValidatorEncrypted":"v2:UkNMT05FAABEV3wkWimmVM_myUXfm2wHc95EcEnaA8mjJZ-uPsUq4O5YKnPHB_n4B5imt2SpHEpwHlStL1F7QSEEJZw6U9cU1UZb-kmwYP45FI5C","added":1687355910628}"#;

    fn get_remote_auth(
        on_request: Box<dyn Fn(HttpRequest) -> Result<MockHttpResponse, HttpError> + Send + Sync>,
        auth_provider: MockAuthProvider,
    ) -> Remote {
        let base_url = String::from("https://app.koofr.net");
        let http_client: Arc<Box<dyn HttpClient + Send + Sync>> =
            Arc::new(Box::new(MockHttpClient::new(on_request)));
        let auth_provider: Arc<Box<dyn AuthProvider + Send + Sync>> =
            Arc::new(Box::new(auth_provider));

        Remote::new(base_url, http_client, auth_provider)
    }

    fn get_remote(
        on_request: Box<dyn Fn(HttpRequest) -> Result<MockHttpResponse, HttpError> + Send + Sync>,
    ) -> Remote {
        get_remote_auth(on_request, MockAuthProvider::default())
    }

    #[test]
    fn test_get_user() {
        let remote = get_remote(Box::new(|_| Ok(MockHttpResponse::json(200, USER_JSON))));

        let user = block_on(async { remote.get_user().await }).unwrap();

        assert_eq!(
            user,
            models::User {
                id: String::from("30bce243-bae7-40d3-9f6f-782fb060c3e7"),
                first_name: String::from("Test"),
                last_name: String::from("User"),
                email: String::from("test@example.com"),
                level: 1000,
                has_password: true,
                ..Default::default()
            }
        )
    }

    #[test]
    fn test_get_list_recursive() {
        let remote = get_remote(Box::new(|_| {
            Ok(MockHttpResponse::new(
                200,
                HeaderMap::new(),
                String::from(
r#"{"type":"file","path":"/","file":{"name":"Vault","type":"dir","modified":1677861215152,"size":0,"contentType":"","tags":{}}}
{"type":"file","path":"/test.txt","file":{"name":"test.txt","type":"file","modified":1677861599216,"size":5,"contentType":"text/plain","hash":"2eedb741f199ecc19f1ba815d3d9914d","tags":{}}}
{"type":"error","path":"/error","error":{"code":"Other","message":"Internal error"}}
{"type":"error","error":{"code":"DeviceOffline","message":"Device is offline"}}
"#).into_bytes(),
            ))
        }));

        block_on(async {
            let mut items_stream = remote
                .get_list_recursive(
                    &MountId("86f2d1a7-226e-433e-a9fa-7779392b20fd".into()),
                    &RemotePath("/Vault".into()),
                )
                .await
                .unwrap();

            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::File {
                    path: RemotePath("/".into()),
                    file: models::FilesFile {
                        name: RemoteName("Vault".into()),
                        typ: String::from("dir"),
                        modified: 1677861215152,
                        size: 0,
                        content_type: String::from(""),
                        hash: None,
                        tags: HashMap::new(),
                    }
                }))
            );
            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::File {
                    path: RemotePath("/test.txt".into()),
                    file: models::FilesFile {
                        name: RemoteName("test.txt".into()),
                        typ: String::from("file"),
                        modified: 1677861599216,
                        size: 5,
                        content_type: String::from("text/plain"),
                        hash: Some(String::from("2eedb741f199ecc19f1ba815d3d9914d")),
                        tags: HashMap::new(),
                    }
                }))
            );
            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::Error {
                    path: Some(RemotePath("/error".into())),
                    error: models::ApiErrorDetails {
                        code: String::from("Other"),
                        message: String::from("Internal error"),
                        extra: None
                    }
                }))
            );
            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::Error {
                    path: None,
                    error: models::ApiErrorDetails {
                        code: String::from("DeviceOffline"),
                        message: String::from("Device is offline"),
                        extra: None
                    }
                }))
            );
            assert_eq!(items_stream.next().await, None);
        });
    }

    #[test]
    fn test_get_list_recursive_network_error() {
        let remote = get_remote(Box::new(|_| {
            let mut res = MockHttpResponse::new(200, HeaderMap::new(), Vec::new());
            res.bytes_stream = Some(Box::pin(stream::once(async {
                Ok(String::from(r#"{"type":"file","path":"/","file":{"name":"Vault","type":"dir","modified":1677861215152,"size":0,"contentType":"","tags":{}}}
"#).into_bytes())
            }).chain(stream::once(async {
                Err(HttpError::ResponseError(String::from("some network error")))
            }))));
            Ok(res)
        }));

        block_on(async {
            let mut items_stream = remote
                .get_list_recursive(
                    &MountId("86f2d1a7-226e-433e-a9fa-7779392b20fd".into()),
                    &RemotePath("/Vault".into()),
                )
                .await
                .unwrap();

            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::File {
                    path: RemotePath("/".into()),
                    file: models::FilesFile {
                        name: RemoteName("Vault".into()),
                        typ: String::from("dir"),
                        modified: 1677861215152,
                        size: 0,
                        content_type: String::from(""),
                        hash: None,
                        tags: HashMap::new(),
                    }
                }))
            );
            assert_eq!(
                items_stream.next().await,
                Some(Err(RemoteError::HttpError(HttpError::ResponseError(
                    String::from("some network error")
                ))))
            );
            assert_eq!(items_stream.next().await, None);
        });
    }

    #[test]
    fn test_retry_network_error_ok() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote(Box::new(move |_| {
            let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

            match attempt {
                0 => Err(HttpError::ResponseError("network error".into())),
                1 => Ok(MockHttpResponse::json(200, USER_JSON)),
                _ => panic!("only one retry"),
            }
        }));

        block_on(async { remote.get_user().await }).unwrap();
    }

    #[test]
    fn test_retry_network_error_fail() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote(Box::new(move |_| {
            let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

            match attempt {
                0 => Err(HttpError::ResponseError("network error".into())),
                1 => Err(HttpError::ResponseError("also network error".into())),
                _ => panic!("only one retry"),
            }
        }));

        let res = block_on(async { remote.get_user().await });

        assert_eq!(
            res.unwrap_err().to_string(),
            "response error: network error"
        );
    }

    fn counter_auth_provider(force_refresh_token_counter: Arc<AtomicUsize>) -> MockAuthProvider {
        MockAuthProvider::new(Box::new(move |force_refresh_token| {
            let count = force_refresh_token_counter.fetch_add(
                if force_refresh_token { 1 } else { 0 },
                std::sync::atomic::Ordering::SeqCst,
            );

            Ok(format!("Bearer TOKEN{}", count))
        }))
    }

    #[test]
    fn test_retry_expired_oauth_token_refresh() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote_auth(
            Box::new(move |request| {
                let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

                match attempt {
                    0 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN0"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    1 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN1"
                        );

                        Ok(MockHttpResponse::json(200, USER_JSON))
                    }
                    _ => panic!("only one retry"),
                }
            }),
            counter_auth_provider(Arc::new(AtomicUsize::new(0))),
        );

        block_on(async { remote.get_user().await }).unwrap();
    }

    #[test]
    fn test_retry_expired_oauth_token_error() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote_auth(
            Box::new(move |request| {
                let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

                match attempt {
                    0 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN0"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    1 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN1"
                        );

                        Err(HttpError::ResponseError("network error".into()))
                    }
                    _ => panic!("only one retry"),
                }
            }),
            counter_auth_provider(Arc::new(AtomicUsize::new(0))),
        );

        let res = block_on(async { remote.get_user().await });

        assert_eq!(res.unwrap_err().to_string(), "unexpected status: 401: ");
    }

    #[test]
    fn test_retry_expired_oauth_token_expired() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote_auth(
            Box::new(move |request| {
                let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

                match attempt {
                    0 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN0"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    1 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN1"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    _ => panic!("only one retry"),
                }
            }),
            counter_auth_provider(Arc::new(AtomicUsize::new(0))),
        );

        let res = block_on(async { remote.get_user().await });

        assert_eq!(res.unwrap_err().to_string(), "unexpected status: 401: ");
    }

    #[test]
    fn test_retry_revoked_oauth_token() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote_auth(
            Box::new(move |request| {
                let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

                match attempt {
                    0 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN0"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    1 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN0"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    _ => panic!("only one retry"),
                }
            }),
            MockAuthProvider::new(Box::new(move |force_refresh_token| {
                if force_refresh_token {
                    Err(AuthError::OAuth2Error(OAuth2Error::InvalidGrant(
                        "invalid grant".into(),
                    )))
                } else {
                    Ok("Bearer TOKEN0".into())
                }
            })),
        );

        let res = block_on(async { remote.get_user().await });

        assert_eq!(res.unwrap_err().to_string(), "unexpected status: 401: ");
    }

    #[test]
    fn test_retry_non_retriable_network_error_fail() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote(Box::new(move |_| {
            let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

            match attempt {
                0 => Err(HttpError::ResponseError("network error".into())),
                _ => panic!("non-retriable"),
            }
        }));

        let res = block_on(async {
            remote
                .create_vault_repo(models::VaultRepoCreate::default())
                .await
        });

        assert_eq!(
            res.unwrap_err().to_string(),
            "response error: network error"
        );
    }

    #[test]
    fn test_retry_non_retriable_expired_oauth_token_refresh() {
        let request_counter = Arc::new(AtomicUsize::new(0));

        let remote_request_counter = request_counter.clone();
        let remote = get_remote_auth(
            Box::new(move |request| {
                let attempt = remote_request_counter.fetch_add(1, Ordering::SeqCst);

                match attempt {
                    0 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN0"
                        );

                        Ok(MockHttpResponse::new(401, HeaderMap::new(), vec![]))
                    }
                    1 => {
                        assert_eq!(
                            request.headers.get("Authorization").unwrap(),
                            "Bearer TOKEN1"
                        );

                        Ok(MockHttpResponse::new(
                            201,
                            HeaderMap::new(),
                            String::from(VAULT_REPO_JSON).into_bytes(),
                        ))
                    }
                    _ => panic!("non-retriable"),
                }
            }),
            counter_auth_provider(Arc::new(AtomicUsize::new(0))),
        );

        block_on(async {
            remote
                .create_vault_repo(models::VaultRepoCreate::default())
                .await
        })
        .unwrap();
    }
}
