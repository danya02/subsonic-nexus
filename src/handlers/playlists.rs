use std::time::Duration;

use axum::extract::State;
use opensubsonic::data::{PlayQueue, PlayQueueByIndex, Playlist, PlaylistWithSongs};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::config::ServerConfig;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::{IdTemplate, build_upstream_url, parse_entity_id};
use crate::response::{Empty, SubsonicResponse};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct PlaylistsBody {
    pub playlist: Vec<Playlist>,
}

#[derive(Serialize)]
pub struct PlaylistsResponse {
    pub playlists: PlaylistsBody,
}

/// Used by getPlaylist and createPlaylist.
#[derive(Serialize)]
pub struct PlaylistResponse {
    pub playlist: PlaylistWithSongs,
}

#[derive(Serialize)]
pub struct PlayQueueResponse {
    #[serde(rename = "playQueue")]
    pub play_queue: PlayQueue,
}

#[derive(Serialize)]
pub struct PlayQueueByIndexResponse {
    #[serde(rename = "playQueueByIndex")]
    pub play_queue_by_index: PlayQueueByIndex,
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Resolve the write_target server config, or return `not_authorized`.
fn write_target_cfg<'a>(
    state: &'a AppState,
) -> Result<&'a ServerConfig, SubsonicError> {
    let name = state.config.nexus.write_target.as_deref().ok_or_else(|| {
        SubsonicError::not_authorized("No write_target configured; playlist operations are read-only")
    })?;
    state.config.server_by_name(name).ok_or_else(|| {
        SubsonicError::not_authorized(format!("write_target server '{name}' not found in config"))
    })
}

/// Translate a slice of nexus song IDs to upstream IDs using the configured template.
fn translate_ids(template: IdTemplate, nexus_ids: &[String]) -> Vec<String> {
    nexus_ids
        .iter()
        .map(|id| parse_entity_id(template, id).upstream_id)
        .collect()
}

/// Build an authenticated URL and issue a GET request, returning the raw
/// JSON response body.
async fn proxy_get(
    server_cfg: &ServerConfig,
    endpoint: &str,
    params: &[(&str, &str)],
) -> Result<serde_json::Value, SubsonicError> {
    let url = build_upstream_url(server_cfg, endpoint, params);
    tracing::info!(endpoint, server = %server_cfg.name, "playlist proxy: sending");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(endpoint, server = %server_cfg.name, error = %e, "playlist proxy: request failed");
            SubsonicError::generic(format!("Upstream request failed: {e}"))
        })?;
    let http_status = resp.status();
    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| {
            tracing::warn!(endpoint, server = %server_cfg.name, %http_status, error = %e, "playlist proxy: parse error");
            SubsonicError::generic(format!("Failed to parse upstream response: {e}"))
        })?;
    tracing::info!(endpoint, server = %server_cfg.name, %http_status, "playlist proxy: ok");
    Ok(body)
}

/// Extract the inner response object from a Subsonic JSON envelope, checking
/// for error status.
fn subsonic_inner(body: serde_json::Value) -> Result<serde_json::Value, SubsonicError> {
    let resp = body
        .get("subsonic-response")
        .cloned()
        .ok_or_else(|| SubsonicError::generic("Unexpected upstream response format"))?;

    if resp.get("status").and_then(|s| s.as_str()) != Some("ok") {
        let msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("upstream error")
            .to_owned();
        return Err(SubsonicError::generic(msg));
    }
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- getPlaylists ---

#[derive(Deserialize)]
pub struct GetPlaylistsParams {
    pub username: Option<String>,
}

/// GET/POST /rest/getPlaylists
///
/// If a `write_target` is configured, proxy the call to that server and
/// return its playlists.  Otherwise return an empty list.
pub async fn get_playlists(
    auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetPlaylistsParams>,
) -> Result<SubsonicResponse<PlaylistsResponse>, SubsonicError> {
    let Ok(server_cfg) = write_target_cfg(&state) else {
        return Ok(PlaylistsResponse { playlists: PlaylistsBody { playlist: vec![] } }.into());
    };

    let username_s;
    let mut extra: Vec<(&str, &str)> = Vec::new();
    let u = params.username.as_deref().or(auth.username.as_deref());
    if let Some(u) = u {
        username_s = u.to_owned();
        extra.push(("username", &username_s));
    }

    let body = proxy_get(server_cfg, "getPlaylists", &extra).await?;
    let resp = subsonic_inner(body)?;

    let playlists: Vec<Playlist> = resp
        .get("playlists")
        .and_then(|p| p.get("playlist"))
        .and_then(|a| serde_json::from_value(a.clone()).ok())
        .unwrap_or_default();

    Ok(PlaylistsResponse { playlists: PlaylistsBody { playlist: playlists } }.into())
}

// --- getPlaylist ---

#[derive(Deserialize)]
pub struct GetPlaylistParams {
    pub id: String,
}

/// GET/POST /rest/getPlaylist
pub async fn get_playlist(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetPlaylistParams>,
) -> Result<SubsonicResponse<PlaylistResponse>, SubsonicError> {
    let server_cfg = write_target_cfg(&state)?;

    let body = proxy_get(server_cfg, "getPlaylist", &[("id", &params.id)]).await?;
    let resp = subsonic_inner(body)?;

    let playlist: PlaylistWithSongs = resp
        .get("playlist")
        .ok_or_else(|| SubsonicError::not_found(format!("Playlist not found: {}", params.id)))
        .and_then(|p| {
            serde_json::from_value(p.clone())
                .map_err(|e| SubsonicError::generic(e.to_string()))
        })?;

    Ok(PlaylistResponse { playlist }.into())
}

// --- createPlaylist ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlaylistParams {
    pub playlist_id: Option<String>,
    pub name: Option<String>,
    #[serde(default)]
    pub song_id: Vec<String>,
}

/// GET/POST /rest/createPlaylist
pub async fn create_playlist(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<CreatePlaylistParams>,
) -> Result<SubsonicResponse<PlaylistResponse>, SubsonicError> {
    let server_cfg = write_target_cfg(&state)?;
    let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);

    let upstream_song_ids = translate_ids(template, &params.song_id);
    let name_s = params.name.as_deref().unwrap_or("");
    let playlist_id_s = params.playlist_id.as_deref().unwrap_or("");

    // Build param list. song_id is multi-valued so we need repeated key.
    let song_pairs: Vec<(String, String)> = upstream_song_ids
        .iter()
        .map(|id| ("songId".to_owned(), id.clone()))
        .collect();

    let mut extra: Vec<(&str, &str)> = Vec::new();
    if !name_s.is_empty() {
        extra.push(("name", name_s));
    }
    if !playlist_id_s.is_empty() {
        extra.push(("playlistId", playlist_id_s));
    }
    for (k, v) in &song_pairs {
        extra.push((k.as_str(), v.as_str()));
    }

    let body = proxy_get(server_cfg, "createPlaylist", &extra).await?;
    let resp = subsonic_inner(body)?;

    let playlist: PlaylistWithSongs = resp
        .get("playlist")
        .ok_or_else(|| SubsonicError::generic("No playlist in upstream response"))
        .and_then(|p| {
            serde_json::from_value(p.clone())
                .map_err(|e| SubsonicError::generic(e.to_string()))
        })?;

    Ok(PlaylistResponse { playlist }.into())
}

// --- updatePlaylist ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlaylistParams {
    pub playlist_id: String,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub public: Option<bool>,
    #[serde(default)]
    pub song_id_to_add: Vec<String>,
    #[serde(default)]
    pub song_index_to_remove: Vec<i32>,
}

/// GET/POST /rest/updatePlaylist
pub async fn update_playlist(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<UpdatePlaylistParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    let server_cfg = write_target_cfg(&state)?;
    let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);

    let upstream_add_ids = translate_ids(template, &params.song_id_to_add);
    let remove_strs: Vec<String> =
        params.song_index_to_remove.iter().map(|i| i.to_string()).collect();

    let name_s = params.name.as_deref().unwrap_or("");
    let comment_s = params.comment.as_deref().unwrap_or("");
    let public_s = params.public.map(|b| b.to_string()).unwrap_or_default();

    let add_pairs: Vec<(String, String)> = upstream_add_ids
        .iter()
        .map(|id| ("songIdToAdd".to_owned(), id.clone()))
        .collect();
    let remove_pairs: Vec<(String, String)> = remove_strs
        .iter()
        .map(|i| ("songIndexToRemove".to_owned(), i.clone()))
        .collect();

    let mut extra: Vec<(&str, &str)> = vec![("playlistId", &params.playlist_id)];
    if !name_s.is_empty() {
        extra.push(("name", name_s));
    }
    if !comment_s.is_empty() {
        extra.push(("comment", comment_s));
    }
    if !public_s.is_empty() {
        extra.push(("public", &public_s));
    }
    for (k, v) in &add_pairs {
        extra.push((k.as_str(), v.as_str()));
    }
    for (k, v) in &remove_pairs {
        extra.push((k.as_str(), v.as_str()));
    }

    let body = proxy_get(server_cfg, "updatePlaylist", &extra).await?;
    subsonic_inner(body)?;

    Ok(Empty {}.into())
}

// --- deletePlaylist ---

#[derive(Deserialize)]
pub struct DeletePlaylistParams {
    pub id: String,
}

/// GET/POST /rest/deletePlaylist
pub async fn delete_playlist(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<DeletePlaylistParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    let server_cfg = write_target_cfg(&state)?;

    let body = proxy_get(server_cfg, "deletePlaylist", &[("id", &params.id)]).await?;
    subsonic_inner(body)?;

    Ok(Empty {}.into())
}

// ---------------------------------------------------------------------------
// Play queue
// ---------------------------------------------------------------------------

/// GET/POST /rest/getPlayQueue — no extra parameters
pub async fn get_play_queue(auth: SubsonicAuth) -> SubsonicResponse<PlayQueueResponse> {
    let username = auth.username.unwrap_or_else(|| "admin".to_string());
    PlayQueueResponse {
        play_queue: PlayQueue {
            current: None,
            position: None,
            username,
            changed: "1970-01-01T00:00:00".to_string(),
            changed_by: "nexus".to_string(),
            entry: vec![],
        },
    }
    .into()
}

// --- savePlayQueue ---

#[derive(Deserialize)]
pub struct SavePlayQueueParams {
    #[serde(default)]
    pub id: Vec<String>,
    pub current: Option<String>,
    pub position: Option<i64>,
}

/// GET/POST /rest/savePlayQueue
///
/// Proxied to write_target if configured; silently accepted otherwise.
pub async fn save_play_queue(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<SavePlayQueueParams>,
) -> SubsonicResponse<Empty> {
    if let Ok(server_cfg) = write_target_cfg(&state) {
        let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);
        let upstream_ids = translate_ids(template, &params.id);
        let current_s = params.current.clone().unwrap_or_default();
        let position_s = params.position.map(|p| p.to_string()).unwrap_or_default();

        let id_pairs: Vec<(String, String)> = upstream_ids
            .iter()
            .map(|id| ("id".to_owned(), id.clone()))
            .collect();

        let mut extra: Vec<(&str, &str)> = Vec::new();
        if !current_s.is_empty() {
            extra.push(("current", &current_s));
        }
        if !position_s.is_empty() {
            extra.push(("position", &position_s));
        }
        for (k, v) in &id_pairs {
            extra.push((k.as_str(), v.as_str()));
        }

        let url = build_upstream_url(server_cfg, "savePlayQueue", &extra);
        tracing::info!(server = %server_cfg.name, "playlist proxy: savePlayQueue sending");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        match client.get(&url).send().await {
            Ok(r) => tracing::info!(server = %server_cfg.name, status = %r.status(), "playlist proxy: savePlayQueue ok"),
            Err(e) => tracing::warn!(server = %server_cfg.name, error = %e, "playlist proxy: savePlayQueue failed"),
        }
    }
    Empty {}.into()
}

/// GET/POST /rest/getPlayQueueByIndex — no extra parameters
pub async fn get_play_queue_by_index(
    auth: SubsonicAuth,
) -> SubsonicResponse<PlayQueueByIndexResponse> {
    let username = auth.username.unwrap_or_else(|| "admin".to_string());
    PlayQueueByIndexResponse {
        play_queue_by_index: PlayQueueByIndex {
            current_index: None,
            position: None,
            username,
            changed: "1970-01-01T00:00:00".to_string(),
            changed_by: "nexus".to_string(),
            entry: vec![],
        },
    }
    .into()
}

// --- savePlayQueueByIndex ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePlayQueueByIndexParams {
    #[serde(default)]
    pub id: Vec<String>,
    pub current_index: Option<i32>,
    pub position: Option<i64>,
}

/// GET/POST /rest/savePlayQueueByIndex
pub async fn save_play_queue_by_index(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<SavePlayQueueByIndexParams>,
) -> SubsonicResponse<Empty> {
    if let Ok(server_cfg) = write_target_cfg(&state) {
        let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);
        let upstream_ids = translate_ids(template, &params.id);
        let current_index_s = params.current_index.map(|i| i.to_string()).unwrap_or_default();
        let position_s = params.position.map(|p| p.to_string()).unwrap_or_default();

        let id_pairs: Vec<(String, String)> = upstream_ids
            .iter()
            .map(|id| ("id".to_owned(), id.clone()))
            .collect();

        let mut extra: Vec<(&str, &str)> = Vec::new();
        if !current_index_s.is_empty() {
            extra.push(("currentIndex", &current_index_s));
        }
        if !position_s.is_empty() {
            extra.push(("position", &position_s));
        }
        for (k, v) in &id_pairs {
            extra.push((k.as_str(), v.as_str()));
        }

        let url = build_upstream_url(server_cfg, "savePlayQueueByIndex", &extra);
        tracing::info!(server = %server_cfg.name, "playlist proxy: savePlayQueueByIndex sending");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        match client.get(&url).send().await {
            Ok(r) => tracing::info!(server = %server_cfg.name, status = %r.status(), "playlist proxy: savePlayQueueByIndex ok"),
            Err(e) => tracing::warn!(server = %server_cfg.name, error = %e, "playlist proxy: savePlayQueueByIndex failed"),
        }
    }
    Empty {}.into()
}
