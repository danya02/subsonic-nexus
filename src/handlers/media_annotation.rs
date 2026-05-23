use serde::Deserialize;

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// --- star / unstar ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarParams {
    #[serde(default)]
    pub id: Vec<String>,
    #[serde(default)]
    pub album_id: Vec<String>,
    #[serde(default)]
    pub artist_id: Vec<String>,
}

/// GET/POST /rest/star
pub async fn star(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<StarParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

/// GET/POST /rest/unstar
pub async fn unstar(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<StarParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- setRating ---

#[derive(Deserialize)]
pub struct SetRatingParams {
    pub id: String,
    /// 0 to remove rating; 1–5 to set it.
    pub rating: i32,
}

/// GET/POST /rest/setRating
pub async fn set_rating(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<SetRatingParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- scrobble ---

#[derive(Deserialize)]
pub struct ScrobbleParams {
    /// Repeated for batch scrobbles.
    pub id: String,
    pub time: Option<i64>,
    pub submission: Option<bool>,
}

/// GET/POST /rest/scrobble
pub async fn scrobble(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<ScrobbleParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- reportPlayback ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPlaybackParams {
    pub media_id: String,
    pub media_type: String,
    pub position_ms: i64,
    pub state: String,
    pub ignore_scrobble: Option<bool>,
    pub playback_rate: Option<f32>,
}

/// GET/POST /rest/reportPlayback
pub async fn report_playback(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<ReportPlaybackParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}
