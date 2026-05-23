use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getScanStatus
pub async fn get_scan_status(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/startScan
pub async fn start_scan(_auth: SubsonicAuth) -> Response {
    todo!()
}
