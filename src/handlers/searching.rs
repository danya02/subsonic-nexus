use opensubsonic::data::{SearchResult, SearchResult2, SearchResult3};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::SubsonicResponse;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SearchResultResponse {
    #[serde(rename = "searchResult")]
    pub search_result: SearchResult,
}

#[derive(Serialize)]
pub struct SearchResult2Response {
    #[serde(rename = "searchResult2")]
    pub search_result2: SearchResult2,
}

#[derive(Serialize)]
pub struct SearchResult3Response {
    #[serde(rename = "searchResult3")]
    pub search_result3: SearchResult3,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- search (legacy) ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    /// `any` in the spec; set to true to search all fields.
    pub any: Option<bool>,
    pub count: Option<i32>,
    pub offset: Option<i32>,
    pub newer_than: Option<i64>,
}

/// GET/POST /rest/search
pub async fn search(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<SearchParams>,
) -> SubsonicResponse<SearchResultResponse> {
    todo!()
}

// --- search2 / search3 ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Search2Params {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/search2
pub async fn search2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<Search2Params>,
) -> SubsonicResponse<SearchResult2Response> {
    todo!()
}

/// GET/POST /rest/search3
pub async fn search3(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<Search2Params>,
) -> SubsonicResponse<SearchResult3Response> {
    todo!()
}
