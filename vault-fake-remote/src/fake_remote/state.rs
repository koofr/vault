use std::collections::HashMap;

use vault_core::remote::models;

use super::files::Filesystem;

#[derive(Debug)]
pub struct UserContainer {
    pub user: models::User,
    // mount ids
    pub mounts: Vec<String>,
    // vault repo ids
    pub user_vault_repos: Vec<String>,
}

#[derive(Debug, Default)]
pub struct FakeRemoteState {
    pub default_user_id: Option<String>,
    pub users: HashMap<String, UserContainer>,

    /// access tokens to user ids
    pub oauth2_access_tokens: HashMap<String, String>,
    /// refresh tokens to user ids
    pub oauth2_refresh_tokens: HashMap<String, String>,
    /// codes to refresh tokens
    pub oauth2_codes: HashMap<String, String>,

    pub mounts: HashMap<String, models::Mount>,

    pub vault_repos: HashMap<String, models::VaultRepo>,

    // mount ids to filesystems
    pub filesystems: HashMap<String, Filesystem>,
}
