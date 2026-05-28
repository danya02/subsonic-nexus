use axum::response::{IntoResponse, Response};
use opensubsonic::data::TranscodeDecision;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::response::SubsonicResponse;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct TranscodeDecisionResponse {
    #[serde(rename = "transcodeDecision")]
    pub transcode_decision: TranscodeDecision,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- getTranscodeDecision ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTranscodeDecisionParams {
    pub media_id: String,
    pub media_type: String,
}

/// POST /rest/getTranscodeDecision (POST only per spec)
pub async fn get_transcode_decision(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetTranscodeDecisionParams>,
) -> Result<SubsonicResponse<TranscodeDecisionResponse>, SubsonicError> {
    Err(SubsonicError::not_found("Transcoding is not supported"))
}

// --- getTranscodeStream ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTranscodeStreamParams {
    pub media_id: String,
    pub media_type: String,
    pub transcode_params: String,
    pub offset: Option<i64>,
}

/// GET /rest/getTranscodeStream (GET only per spec) — returns a binary stream.
pub async fn get_transcode_stream(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetTranscodeStreamParams>,
) -> Response {
    SubsonicError::not_found("Transcoding is not supported").into_response()
}
