use opensubsonic::data::{PlayQueue, PlayQueueByIndex, Playlist, PlaylistWithSongs};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

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
// Handlers
// ---------------------------------------------------------------------------

// --- getPlaylists ---

#[derive(Deserialize)]
pub struct GetPlaylistsParams {
    pub username: Option<String>,
}

/// GET/POST /rest/getPlaylists
pub async fn get_playlists(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetPlaylistsParams>,
) -> SubsonicResponse<PlaylistsResponse> {
    todo!()
}

// --- getPlaylist ---

#[derive(Deserialize)]
pub struct GetPlaylistParams {
    pub id: String,
}

/// GET/POST /rest/getPlaylist
pub async fn get_playlist(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetPlaylistParams>,
) -> SubsonicResponse<PlaylistResponse> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<CreatePlaylistParams>,
) -> SubsonicResponse<PlaylistResponse> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<UpdatePlaylistParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- deletePlaylist ---

#[derive(Deserialize)]
pub struct DeletePlaylistParams {
    pub id: String,
}

/// GET/POST /rest/deletePlaylist
pub async fn delete_playlist(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeletePlaylistParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

/// GET/POST /rest/getPlayQueue — no extra parameters
pub async fn get_play_queue(_auth: SubsonicAuth) -> SubsonicResponse<PlayQueueResponse> {
    todo!()
}

// --- savePlayQueue ---

#[derive(Deserialize)]
pub struct SavePlayQueueParams {
    /// Repeated for each song in the queue.
    #[serde(default)]
    pub id: Vec<String>,
    pub current: Option<String>,
    pub position: Option<i64>,
}

/// GET/POST /rest/savePlayQueue
pub async fn save_play_queue(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<SavePlayQueueParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

/// GET/POST /rest/getPlayQueueByIndex — no extra parameters
pub async fn get_play_queue_by_index(
    _auth: SubsonicAuth,
) -> SubsonicResponse<PlayQueueByIndexResponse> {
    todo!()
}

// --- savePlayQueueByIndex ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePlayQueueByIndexParams {
    /// Repeated for each song in the queue.
    #[serde(default)]
    pub id: Vec<String>,
    pub current_index: Option<i32>,
    pub position: Option<i64>,
}

/// GET/POST /rest/savePlayQueueByIndex
pub async fn save_play_queue_by_index(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<SavePlayQueueByIndexParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}
