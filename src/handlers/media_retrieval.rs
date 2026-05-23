use axum::response::Response;
use opensubsonic::data::{Lyrics, LyricsList};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::SubsonicResponse;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct LyricsResponse {
    pub lyrics: Lyrics,
}

#[derive(Serialize)]
pub struct LyricsListResponse {
    #[serde(rename = "lyricsList")]
    pub lyrics_list: LyricsList,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- stream ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamParams {
    pub id: String,
    pub max_bit_rate: Option<i32>,
    pub format: Option<String>,
    pub time_offset: Option<i32>,
    /// Requested video size as "WxH", e.g. "640x480".
    pub size: Option<String>,
    pub estimate_content_length: Option<bool>,
    pub converted: Option<bool>,
}

/// GET/POST /rest/stream — returns a binary audio/video stream.
pub async fn stream(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<StreamParams>,
) -> Response {
    todo!()
}

// --- download ---

#[derive(Deserialize)]
pub struct DownloadParams {
    pub id: String,
}

/// GET/POST /rest/download — returns a binary file download.
pub async fn download(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DownloadParams>,
) -> Response {
    todo!()
}

// --- getCoverArt ---

#[derive(Deserialize)]
pub struct GetCoverArtParams {
    pub id: String,
    pub size: Option<i32>,
}

/// GET/POST /rest/getCoverArt — returns raw image bytes.
pub async fn get_cover_art(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetCoverArtParams>,
) -> Response {
    todo!()
}

// --- getLyrics ---

#[derive(Deserialize)]
pub struct GetLyricsParams {
    pub artist: Option<String>,
    pub title: Option<String>,
}

/// GET/POST /rest/getLyrics
pub async fn get_lyrics(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetLyricsParams>,
) -> SubsonicResponse<LyricsResponse> {
    todo!()
}

// --- getLyricsBySongId ---

#[derive(Deserialize)]
pub struct GetLyricsBySongIdParams {
    pub id: String,
    pub enhanced: Option<bool>,
}

/// GET/POST /rest/getLyricsBySongId
pub async fn get_lyrics_by_song_id(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetLyricsBySongIdParams>,
) -> SubsonicResponse<LyricsListResponse> {
    todo!()
}

// --- getAvatar ---

#[derive(Deserialize)]
pub struct GetAvatarParams {
    pub username: String,
}

/// GET/POST /rest/getAvatar — returns raw image bytes.
pub async fn get_avatar(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetAvatarParams>,
) -> Response {
    todo!()
}

// --- getCaptions ---

#[derive(Deserialize)]
pub struct GetCaptionsParams {
    pub id: String,
    pub format: Option<String>,
}

/// GET/POST /rest/getCaptions — returns a subtitle file.
pub async fn get_captions(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetCaptionsParams>,
) -> Response {
    todo!()
}

// --- hls.m3u8 ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HlsParams {
    pub id: String,
    pub bit_rate: Option<i32>,
    pub audio_track: Option<String>,
}

/// GET/POST /rest/hls.m3u8 — returns an HLS playlist.
pub async fn hls(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<HlsParams>,
) -> Response {
    todo!()
}
