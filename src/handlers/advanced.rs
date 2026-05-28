use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- findSonicPath ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindSonicPathParams {
    pub start_song_id: String,
    pub end_song_id: String,
    pub count: Option<i32>,
}

/// GET/POST /rest/findSonicPath
///
/// Response shape not yet defined in the `opensubsonic` crate; returns a raw
/// Response until an appropriate type is available.
pub async fn find_sonic_path(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<FindSonicPathParams>,
) -> Response {
    SubsonicError::not_found("findSonicPath is not supported").into_response()
}

// --- getSonicSimilarTracks ---

#[derive(Deserialize)]
pub struct GetSonicSimilarTracksParams {
    pub id: String,
    pub count: Option<i32>,
}

/// GET/POST /rest/getSonicSimilarTracks
///
/// Response shape not yet defined in the `opensubsonic` crate; returns a raw
/// Response until an appropriate type is available.
pub async fn get_sonic_similar_tracks(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetSonicSimilarTracksParams>,
) -> Response {
    SubsonicError::not_found("getSonicSimilarTracks is not supported").into_response()
}
