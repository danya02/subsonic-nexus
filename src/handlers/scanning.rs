use opensubsonic::data::ScanStatus;
use serde::Serialize;

use crate::auth::SubsonicAuth;
use crate::response::SubsonicResponse;

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
// Handlers — no extra parameters for either endpoint
// ---------------------------------------------------------------------------

/// GET/POST /rest/getScanStatus
pub async fn get_scan_status(_auth: SubsonicAuth) -> SubsonicResponse<ScanStatusResponse> {
    todo!()
}

/// GET/POST /rest/startScan
pub async fn start_scan(_auth: SubsonicAuth) -> SubsonicResponse<ScanStatusResponse> {
    todo!()
}
