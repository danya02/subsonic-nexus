use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getShares
pub async fn get_shares(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/createShare
pub async fn create_share(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/updateShare
pub async fn update_share(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deleteShare
pub async fn delete_share(_auth: SubsonicAuth) -> Response {
    todo!()
}
