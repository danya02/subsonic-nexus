use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/stream
/// NOTE: Will return a binary audio stream, not a JSON SubsonicResponse.
pub async fn stream(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/download
/// NOTE: Will return a binary file download, not a JSON SubsonicResponse.
pub async fn download(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getCoverArt
/// NOTE: Will return image bytes, not a JSON SubsonicResponse.
pub async fn get_cover_art(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getLyrics
pub async fn get_lyrics(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getLyricsBySongId
pub async fn get_lyrics_by_song_id(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getAvatar
/// NOTE: Will return image bytes, not a JSON SubsonicResponse.
pub async fn get_avatar(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getCaptions
/// NOTE: Will return a subtitle file, not a JSON SubsonicResponse.
pub async fn get_captions(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/hls.m3u8
/// NOTE: Will return an HLS playlist, not a JSON SubsonicResponse.
pub async fn hls(_auth: SubsonicAuth) -> Response {
    todo!()
}
