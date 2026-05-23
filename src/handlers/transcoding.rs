use axum::response::Response;

use crate::auth::SubsonicAuth;

/// POST /rest/getTranscodeDecision  (POST only per spec)
pub async fn get_transcode_decision(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET /rest/getTranscodeStream  (GET only per spec)
/// NOTE: Will return a binary audio stream, not a JSON SubsonicResponse.
pub async fn get_transcode_stream(_auth: SubsonicAuth) -> Response {
    todo!()
}
