use axum::extract::State;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opensubsonic::data::ScanStatus;
use serde::Serialize;

use crate::auth::SubsonicAuth;
use crate::db::schema::songs;
use crate::error::SubsonicError;
use crate::nexus::get_conn;
use crate::response::SubsonicResponse;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Used by both getScanStatus and startScan.
#[derive(Serialize)]
pub struct ScanStatusResponse {
    #[serde(rename = "scanStatus")]
    pub scan_status: ScanStatus,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn song_count(
    conn: &mut crate::db::AsyncSqliteConnection,
) -> Result<i64, SubsonicError> {
    songs::table
        .count()
        .get_result::<i64>(conn)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))
}

// ---------------------------------------------------------------------------
// Handlers — no extra parameters for either endpoint
// ---------------------------------------------------------------------------

/// GET/POST /rest/getScanStatus
pub async fn get_scan_status(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
) -> Result<SubsonicResponse<ScanStatusResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let count = song_count(&mut conn).await?;
    tracing::debug!(song_count = count, "getScanStatus");
    Ok(ScanStatusResponse {
        scan_status: ScanStatus { scanning: false, count: Some(count) },
    }
    .into())
}

/// GET/POST /rest/startScan
pub async fn start_scan(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
) -> Result<SubsonicResponse<ScanStatusResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let count = song_count(&mut conn).await?;
    drop(conn);

    // Spawn a background rescan.
    let state_clone = state.clone();
    tokio::spawn(async move {
        crate::scanner::run_full_scan(&state_clone.pool, &state_clone.config.servers).await;
    });

    Ok(ScanStatusResponse {
        scan_status: ScanStatus { scanning: true, count: Some(count) },
    }
    .into())
}
