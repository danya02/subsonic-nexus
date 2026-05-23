use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/ping
pub async fn ping(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getLicense
pub async fn get_license(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getOpenSubsonicExtensions
pub async fn get_open_subsonic_extensions(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/tokenInfo
pub async fn token_info(_auth: SubsonicAuth) -> Response {
    todo!()
}
