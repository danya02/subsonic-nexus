use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getPlaylists
pub async fn get_playlists(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getPlaylist
pub async fn get_playlist(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/createPlaylist
pub async fn create_playlist(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/updatePlaylist
pub async fn update_playlist(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deletePlaylist
pub async fn delete_playlist(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getPlayQueue
pub async fn get_play_queue(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/savePlayQueue
pub async fn save_play_queue(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getPlayQueueByIndex
pub async fn get_play_queue_by_index(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/savePlayQueueByIndex
pub async fn save_play_queue_by_index(_auth: SubsonicAuth) -> Response {
    todo!()
}
