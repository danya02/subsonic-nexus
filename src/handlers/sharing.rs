use opensubsonic::data::Share;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SharesBody {
    pub share: Vec<Share>,
}

#[derive(Serialize)]
pub struct SharesResponse {
    pub shares: SharesBody,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET/POST /rest/getShares — no extra parameters
pub async fn get_shares(_auth: SubsonicAuth) -> SubsonicResponse<SharesResponse> {
    SharesResponse {
        shares: SharesBody { share: vec![] },
    }
    .into()
}

// --- createShare ---

#[derive(Deserialize)]
pub struct CreateShareParams {
    #[serde(default)]
    pub id: Vec<String>,
    pub description: Option<String>,
    pub expires: Option<i64>,
}

/// GET/POST /rest/createShare
pub async fn create_share(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<CreateShareParams>,
) -> Result<SubsonicResponse<SharesResponse>, SubsonicError> {
    Err(SubsonicError::not_authorized("Sharing is not supported"))
}

// --- updateShare ---

#[derive(Deserialize)]
pub struct UpdateShareParams {
    pub id: String,
    pub description: Option<String>,
    pub expires: Option<i64>,
}

/// GET/POST /rest/updateShare
pub async fn update_share(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<UpdateShareParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Sharing is not supported"))
}

// --- deleteShare ---

#[derive(Deserialize)]
pub struct DeleteShareParams {
    pub id: String,
}

/// GET/POST /rest/deleteShare
pub async fn delete_share(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeleteShareParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Sharing is not supported"))
}
