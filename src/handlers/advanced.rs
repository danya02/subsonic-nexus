use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/findSonicPath
pub async fn find_sonic_path(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getSonicSimilarTracks
pub async fn get_sonic_similar_tracks(_auth: SubsonicAuth) -> Response {
    todo!()
}
