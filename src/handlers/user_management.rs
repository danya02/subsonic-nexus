use opensubsonic::data::User;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Serialize)]
pub struct UsersBody {
    pub user: Vec<User>,
}

#[derive(Serialize)]
pub struct UsersResponse {
    pub users: UsersBody,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a synthetic admin User for the given username.
fn admin_user(username: String) -> User {
    User {
        username,
        scrobbling_enabled: Some(true),
        max_bit_rate: None,
        admin_role: Some(true),
        settings_role: Some(true),
        download_role: Some(true),
        upload_role: Some(false),
        playlist_role: Some(true),
        cover_art_role: Some(true),
        comment_role: Some(true),
        podcast_role: Some(true),
        stream_role: Some(true),
        jukebox_role: Some(false),
        share_role: Some(false),
        video_conversion_role: Some(false),
        avatar_last_changed: None,
        folder: vec![],
        email: None,
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- getUser ---

#[derive(Deserialize)]
pub struct GetUserParams {
    pub username: String,
}

/// GET/POST /rest/getUser
pub async fn get_user(
    _auth: SubsonicAuth,
    QueryOrForm(params): QueryOrForm<GetUserParams>,
) -> SubsonicResponse<UserResponse> {
    UserResponse { user: admin_user(params.username) }.into()
}

/// GET/POST /rest/getUsers — no extra parameters
pub async fn get_users(auth: SubsonicAuth) -> SubsonicResponse<UsersResponse> {
    let username = auth.username.unwrap_or_else(|| "admin".to_string());
    UsersResponse { users: UsersBody { user: vec![admin_user(username)] } }.into()
}

// --- createUser ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserParams {
    pub username: String,
    pub password: String,
    pub email: String,
    pub ldap_authenticated: Option<bool>,
    pub admin_role: Option<bool>,
    pub settings_role: Option<bool>,
    pub stream_role: Option<bool>,
    pub jukebox_role: Option<bool>,
    pub download_role: Option<bool>,
    pub upload_role: Option<bool>,
    pub playlist_role: Option<bool>,
    pub cover_art_role: Option<bool>,
    pub comment_role: Option<bool>,
    pub podcast_role: Option<bool>,
    pub share_role: Option<bool>,
    pub video_conversion_role: Option<bool>,
    #[serde(default)]
    pub music_folder_id: Vec<String>,
}

/// GET/POST /rest/createUser
pub async fn create_user(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<CreateUserParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("User management is not supported"))
}

// --- updateUser ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserParams {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub ldap_authenticated: Option<bool>,
    pub admin_role: Option<bool>,
    pub settings_role: Option<bool>,
    pub stream_role: Option<bool>,
    pub jukebox_role: Option<bool>,
    pub download_role: Option<bool>,
    pub upload_role: Option<bool>,
    pub playlist_role: Option<bool>,
    pub cover_art_role: Option<bool>,
    pub comment_role: Option<bool>,
    pub podcast_role: Option<bool>,
    pub share_role: Option<bool>,
    pub video_conversion_role: Option<bool>,
    pub max_bit_rate: Option<i32>,
    #[serde(default)]
    pub music_folder_id: Vec<String>,
}

/// GET/POST /rest/updateUser
pub async fn update_user(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<UpdateUserParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("User management is not supported"))
}

// --- deleteUser ---

#[derive(Deserialize)]
pub struct DeleteUserParams {
    pub username: String,
}

/// GET/POST /rest/deleteUser
pub async fn delete_user(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeleteUserParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("User management is not supported"))
}

// --- changePassword ---

#[derive(Deserialize)]
pub struct ChangePasswordParams {
    pub username: String,
    pub password: String,
}

/// GET/POST /rest/changePassword
pub async fn change_password(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<ChangePasswordParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("User management is not supported"))
}
