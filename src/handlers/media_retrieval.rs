use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use opensubsonic::data::{Lyrics, LyricsList};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::{
    IdTemplate, build_upstream_url, get_conn, parse_cover_art_id,
    query_song_by_nexus_id,
};
use crate::response::SubsonicResponse;
use crate::state::AppState;

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
// Shared proxying logic
// ---------------------------------------------------------------------------

/// Build a redirect or proxy response for a media endpoint.
///
/// Looks up the song by nexus id, finds its canonical server config, then
/// either returns an HTTP 302 to the upstream URL or proxies the bytes
/// depending on `state.config.nexus.proxy`.
async fn serve_song_media(
    state: &AppState,
    song_nexus_id: &str,
    endpoint: &str,
    extra_params: &[(&str, &str)],
) -> Result<Response, SubsonicError> {
    tracing::debug!(song_nexus_id, endpoint, "serve_song_media: looking up song");
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let template = IdTemplate::from_config(&cfg.entity_id_template);

    let song = query_song_by_nexus_id(&mut conn, template, song_nexus_id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?
        .ok_or_else(|| {
            tracing::warn!(song_nexus_id, endpoint, "serve_song_media: song not found");
            SubsonicError::not_found(format!("Song not found: {song_nexus_id}"))
        })?;

    // Find the server config.
    let server_cfg = state
        .config
        .servers
        .iter()
        .find(|s| s.name == song.server_name)
        .ok_or_else(|| {
            tracing::warn!(server_name = %song.server_name, "serve_song_media: server not in config");
            SubsonicError::generic(format!(
                "Server '{}' not found in config",
                song.server_name
            ))
        })?;

    let mut params: Vec<(&str, &str)> = vec![("id", &song.upstream_id)];
    params.extend_from_slice(extra_params);
    let url = build_upstream_url(server_cfg, endpoint, &params);

    tracing::debug!(endpoint, server = %server_cfg.name, proxy = cfg.proxy, "serve_song_media: routing to upstream");
    if cfg.proxy {
        proxy_upstream(&url).await
    } else {
        Ok(Redirect::temporary(&url).into_response())
    }
}

/// Proxy an upstream URL, streaming the response body back to the client.
async fn proxy_upstream(url: &str) -> Result<Response, SubsonicError> {
    // Log the host only — the full URL contains auth tokens.
    let host = url.split('/').nth(2).unwrap_or("?");
    tracing::debug!(host, "proxy_upstream: sending request");
    let client = reqwest::Client::new();
    let upstream = client
        .get(url)
        .send()
        .await
        .map_err(|e| SubsonicError::generic(format!("Upstream request failed: {e}")))?;

    let status = StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::OK);
    tracing::debug!(status = upstream.status().as_u16(), "proxy_upstream: response received");
    let mut headers = HeaderMap::new();
    for (name, value) in upstream.headers() {
        if let (Ok(n), Ok(v)) = (
            axum::http::HeaderName::from_bytes(name.as_str().as_bytes()),
            axum::http::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            headers.insert(n, v);
        }
    }

    let body = axum::body::Body::from_stream(upstream.bytes_stream());
    let mut resp = Response::new(body);
    *resp.status_mut() = status;
    *resp.headers_mut() = headers;
    Ok(resp)
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
    pub size: Option<String>,
    pub estimate_content_length: Option<bool>,
    pub converted: Option<bool>,
}

/// GET/POST /rest/stream — returns a binary audio/video stream.
pub async fn stream(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<StreamParams>,
) -> Result<Response, SubsonicError> {
    tracing::info!(song_id = %params.id, max_bit_rate = ?params.max_bit_rate, format = ?params.format, "stream");
    let mut extra: Vec<(&str, &str)> = Vec::new();
    let max_bit_rate_s;
    if let Some(mbr) = params.max_bit_rate {
        max_bit_rate_s = mbr.to_string();
        extra.push(("maxBitRate", &max_bit_rate_s));
    }
    let format_s = params.format.clone().unwrap_or_default();
    if !format_s.is_empty() {
        extra.push(("format", &format_s));
    }
    serve_song_media(&state, &params.id, "stream", &extra).await
}

// --- download ---

#[derive(Deserialize)]
pub struct DownloadParams {
    pub id: String,
}

/// GET/POST /rest/download — returns a binary file download.
pub async fn download(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<DownloadParams>,
) -> Result<Response, SubsonicError> {
    tracing::info!(song_id = %params.id, "download");
    serve_song_media(&state, &params.id, "download", &[]).await
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
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetCoverArtParams>,
) -> Result<Response, SubsonicError> {
    tracing::debug!(id = %params.id, "getCoverArt: parsing cover art id");
    let (server_db_id, upstream_cover_art_id) = parse_cover_art_id(&params.id).ok_or_else(|| {
        tracing::warn!(id = %params.id, "getCoverArt: invalid id (expected {{server_db_id}}:{{upstream_id}} format)");
        SubsonicError::not_found(format!("Invalid cover art id: {}", params.id))
    })?;
    tracing::debug!(server_db_id, upstream_cover_art_id, "getCoverArt: parsed ok");

    // Find server config by DB id.
    let mut conn = get_conn(&state.pool).await?;
    use crate::db::schema::upstream_servers;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    let server_name: String = upstream_servers::table
        .filter(upstream_servers::id.eq(server_db_id))
        .select(upstream_servers::name)
        .first(&mut conn)
        .await
        .map_err(|_| {
            tracing::warn!(server_db_id, "getCoverArt: server db id not found");
            SubsonicError::not_found(format!("Server {server_db_id} not found"))
        })?;

    let server_cfg = state
        .config
        .servers
        .iter()
        .find(|s| s.name == server_name)
        .ok_or_else(|| {
            tracing::warn!(server_name = %server_name, "getCoverArt: server not in config");
            SubsonicError::generic(format!("Server '{server_name}' not in config"))
        })?;

    let size_s = params.size.map(|s| s.to_string());
    let mut cover_params: Vec<(&str, &str)> = vec![("id", upstream_cover_art_id)];
    if let Some(ref s) = size_s {
        cover_params.push(("size", s.as_str()));
    }
    let url = build_upstream_url(server_cfg, "getCoverArt", &cover_params);
    tracing::debug!(server = %server_name, proxy = state.config.nexus.proxy, "getCoverArt: routing to upstream");

    if state.config.nexus.proxy {
        proxy_upstream(&url).await
    } else {
        Ok(Redirect::temporary(&url).into_response())
    }
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
    LyricsResponse {
        lyrics: Lyrics { artist: None, title: None, value: None },
    }
    .into()
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
    LyricsListResponse {
        lyrics_list: LyricsList { structured_lyrics: vec![] },
    }
    .into()
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
) -> Result<Response, SubsonicError> {
    Err(SubsonicError::not_found("Avatar not supported"))
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
) -> Result<Response, SubsonicError> {
    Err(SubsonicError::not_found("Captions not supported"))
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
) -> Result<Response, SubsonicError> {
    Err(SubsonicError::not_found("HLS not supported"))
}
