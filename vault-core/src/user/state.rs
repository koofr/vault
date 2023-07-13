use crate::{common::state::Status, remote::RemoteError};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub email: String,
    pub profile_picture_status: Status<RemoteError>,
    pub profile_picture_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Default)]
pub struct UserState {
    pub status: Status<RemoteError>,
    pub user: Option<User>,
}
