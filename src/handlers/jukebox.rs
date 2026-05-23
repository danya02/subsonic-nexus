use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/jukeboxControl
pub async fn jukebox_control(_auth: SubsonicAuth) -> Response {
    todo!()
}
