use opensubsonic::data::Share;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
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
    todo!()
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
) -> SubsonicResponse<SharesResponse> {
    todo!()
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
) -> SubsonicResponse<Empty> {
    todo!()
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
) -> SubsonicResponse<Empty> {
    todo!()
}
