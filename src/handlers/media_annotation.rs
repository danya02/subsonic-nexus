use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/star
pub async fn star(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/unstar
pub async fn unstar(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/setRating
pub async fn set_rating(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/scrobble
pub async fn scrobble(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/reportPlayback
pub async fn report_playback(_auth: SubsonicAuth) -> Response {
    todo!()
}
