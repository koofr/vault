use std::{
    pin::Pin,
    sync::{Arc, RwLock},
};

use futures::{
    stream::{BoxStream, TryStreamExt},
    AsyncBufReadExt, AsyncRead, StreamExt,
};
use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderMap, HeaderValue,
};
use serde::Serialize;
use urlencoding::encode;

use crate::{
    auth,
    auth::errors::AuthError,
    http::{HttpClient, HttpError, HttpRequest, HttpRequestAbort, HttpRequestBody, HttpResponse},
    oauth2::errors::OAuth2Error,
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
    pub reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
}

pub enum RemoteFileUploadConflictResolution {
    Autorename,
    Overwrite,
    Error,
}

pub type Logout = Box<dyn Fn() + Send + Sync + 'static>;

pub struct Remote {
    base_url: String,
    http_client: Arc<Box<dyn HttpClient + Send + Sync>>,
    auth_provider: Arc<Box<dyn auth::AuthProvider + Send + Sync>>,

    logout: Arc<RwLock<Option<Logout>>>,
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
        }
    }

    pub fn set_logout(&self, logout: Logout) {
        let mut logout_guard = self.logout.write().unwrap();

        *logout_guard = Some(logout)
    }

    async fn request(
        &self,
        request: HttpRequest,
    ) -> Result<Box<dyn HttpResponse + Send + Sync>, RemoteError> {
        let authorization = self.get_authorization(false).await?;

        let mut request = request;

        request.url = format!("{}{}", self.base_url, request.url);

        request
            .headers
            .insert(AUTHORIZATION, authorization.parse().unwrap());

        let res = self
            .http_client
            .request(request)
            .await
            .map_err(RemoteError::HttpError)?;

        // invalid oauth2 token
        if res.status_code() == 401 {
            // try to refresh the token
            let _ = self.get_authorization(true).await;
        }

        Ok(res)
    }

    async fn get_authorization(&self, force_refresh_token: bool) -> Result<String, RemoteError> {
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
                    "/content/api/v2/users/{}/profile-picture?nodefault",
                    user_id,
                ),
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res.bytes().await.map_err(RemoteError::HttpError)
    }

    pub async fn get_mount(&self, id: &str) -> Result<models::Mount, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!("/api/v2.1/mounts/{}", encode(id)),
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
                ..Default::default()
            })
            .await?;

        if res.status_code() != 201 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn remove_vault_repo(&self, repo_id: &str) -> Result<(), RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("DELETE"),
                url: format!("/api/v2.1/vault/repos/{}", repo_id),
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
        mount_id: &str,
        path: &str,
    ) -> Result<models::Bundle, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!("/api/v2.1/mounts/{}/bundle?path={}", mount_id, encode(path)),
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
        mount_id: &str,
        path: &str,
    ) -> Result<ListRecursiveItemStream, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/content/api/v2.1/mounts/{}/files/listrecursive?path={}",
                    mount_id,
                    encode(path)
                ),
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
        mount_id: &str,
        path: &str,
    ) -> Result<models::FilesFile, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/info?path={}",
                    mount_id,
                    encode(path)
                ),
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
        mount_id: &str,
        path: &str,
    ) -> Result<RemoteFileReader, RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("GET"),
                url: format!(
                    "/content/api/v2.1/mounts/{}/files/get?path={}",
                    mount_id,
                    encode(path)
                ),
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
        mount_id: &str,
        parent_path: &str,
        name: &str,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
        size: Option<i64>,
        conflict_resolution: RemoteFileUploadConflictResolution,
        on_progress: Option<Box<dyn Fn(usize) + Send + Sync>>,
        abort: HttpRequestAbort,
    ) -> Result<models::FilesFile, RemoteError> {
        let (autorename, overwrite) = match conflict_resolution {
            RemoteFileUploadConflictResolution::Autorename => (true, false),
            RemoteFileUploadConflictResolution::Overwrite => (false, true),
            RemoteFileUploadConflictResolution::Error => (false, false),
        };

        let mut url = format!(
            "/content/api/v2.1/mounts/{}/files/put?path={}&filename={}&autorename={}&overwrite={}&info=true",
            mount_id,
            encode(parent_path),
            encode(name),
            autorename,
            overwrite,
        );

        if let Some(size) = size {
            url = format!("{}&size={}", url, size);
        }

        let res = self
            .request(HttpRequest {
                method: String::from("POST"),
                url,
                headers: HeaderMap::new(),
                body: Some(HttpRequestBody::Reader(reader)),
                on_body_progress: on_progress,
                abort,
                ..Default::default()
            })
            .await?;

        if res.status_code() != 200 {
            return res_error(res).await;
        }

        res_json(res).await
    }

    pub async fn delete_file(&self, mount_id: &str, path: &str) -> Result<(), RemoteError> {
        let res = self
            .request(HttpRequest {
                method: String::from("DELETE"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/remove?path={}",
                    mount_id,
                    encode(path)
                ),
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
        mount_id: &str,
        parent_path: &str,
        name: &str,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesFolderCreate {
            name: name.to_owned(),
        });

        let res = self
            .request(HttpRequest {
                method: String::from("POST"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/folder?path={}",
                    mount_id,
                    encode(parent_path)
                ),
                headers: req_headers,
                body: req_body,
                on_body_progress: None,
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
        mount_id: &str,
        path: &str,
        new_name: &str,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesRename {
            name: new_name.to_owned(),
        });

        let res = self
            .request(HttpRequest {
                method: String::from("PUT"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/rename?path={}",
                    mount_id,
                    encode(path)
                ),
                headers: req_headers,
                body: req_body,
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
        mount_id: &str,
        path: &str,
        to_mount_id: &str,
        to_path: &str,
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
                    mount_id,
                    encode(path)
                ),
                headers: req_headers,
                body: req_body,
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
        mount_id: &str,
        path: &str,
        to_mount_id: &str,
        to_path: &str,
    ) -> Result<(), RemoteError> {
        let (req_body, req_headers) = req_json(&models::FilesMove {
            to_mount_id: to_mount_id.to_owned(),
            to_path: to_path.to_owned(),
        });

        let res = self
            .request(HttpRequest {
                method: String::from("PUT"),
                url: format!(
                    "/api/v2.1/mounts/{}/files/move?path={}",
                    mount_id,
                    encode(path)
                ),
                headers: req_headers,
                body: req_body,
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
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    (Some(HttpRequestBody::Bytes(body)), headers)
}

async fn res_bytes(res: Box<dyn HttpResponse + Send + Sync>) -> Result<Vec<u8>, RemoteError> {
    res.bytes().await.map_err(RemoteError::HttpError)
}

async fn res_error<T>(res: Box<dyn HttpResponse + Send + Sync>) -> Result<T, RemoteError> {
    let status_code = res.status_code();

    let is_content_type_json = res
        .headers()
        .get(CONTENT_TYPE)
        .filter(|x| x.as_bytes() == b"application/json; charset=utf-8")
        .is_some();

    let bytes = res_bytes(res).await?;

    if is_content_type_json {
        match serde_json::from_slice::<ApiError>(&bytes) {
            Ok(api_error) => return Err(api_error.into()),
            _ => (),
        }
    }

    match status_code {
        404 => {
            return Err(RemoteError::from_code(ApiErrorCode::NotFound, "Not found"));
        }
        _ => (),
    }

    let str = String::from_utf8(bytes).unwrap_or(String::from("non-utf8 response"));

    return Err(RemoteError::HttpError(HttpError::ResponseError(format!(
        "unexpected status: {}: {}",
        status_code, &str,
    ))));
}

async fn res_json<'a, T>(res: Box<dyn HttpResponse + Send + Sync>) -> Result<T, RemoteError>
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

#[cfg(test)]
pub mod tests {
    use std::{collections::HashMap, sync::Arc};

    use futures::{executor::block_on, stream, StreamExt};
    use http::HeaderMap;

    use crate::{
        auth::{mock_auth_provider::MockAuthProvider, AuthProvider},
        http::{
            mock_http_client::{MockHttpClient, MockHttpResponse},
            HttpClient, HttpError, HttpRequest,
        },
        remote::{models, RemoteError},
    };

    use super::Remote;

    fn get_remote(
        on_request: Box<dyn Fn(HttpRequest) -> Result<MockHttpResponse, HttpError> + Send + Sync>,
    ) -> Remote {
        let base_url = String::from("https://app.koofr.net");
        let http_client: Arc<Box<dyn HttpClient + Send + Sync>> =
            Arc::new(Box::new(MockHttpClient::new(on_request)));
        let auth_provider: Arc<Box<dyn AuthProvider + Send + Sync>> =
            Arc::new(Box::new(MockAuthProvider::default()));

        Remote::new(base_url, http_client, auth_provider)
    }

    #[test]
    fn test_get_user() {
        let remote = get_remote(Box::new(|_| {
            Ok(MockHttpResponse::new(
                200,
                HeaderMap::new(),
                String::from(r#"{"id":"30bce243-bae7-40d3-9f6f-782fb060c3e7","firstName":"Test","lastName":"User","email":"test@example.com","level":1000,"hasPassword":true}"#).into_bytes(),
            ))
        }));

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
                .get_list_recursive("86f2d1a7-226e-433e-a9fa-7779392b20fd", "/Vault")
                .await
                .unwrap();

            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::File {
                    path: String::from("/"),
                    file: models::FilesFile {
                        name: String::from("Vault"),
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
                    path: String::from("/test.txt"),
                    file: models::FilesFile {
                        name: String::from("test.txt"),
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
                    path: Some(String::from("/error")),
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
                .get_list_recursive("86f2d1a7-226e-433e-a9fa-7779392b20fd", "/Vault")
                .await
                .unwrap();

            assert_eq!(
                items_stream.next().await,
                Some(Ok(models::FilesListRecursiveItem::File {
                    path: String::from("/"),
                    file: models::FilesFile {
                        name: String::from("Vault"),
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
}
