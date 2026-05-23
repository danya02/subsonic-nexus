use opensubsonic::data::{JukeboxPlaylist, JukeboxStatus};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::SubsonicResponse;

// ---------------------------------------------------------------------------
// Response type
// ---------------------------------------------------------------------------

/// Returns either `{"jukeboxStatus": {...}}` or `{"jukeboxPlaylist": {...}}`
/// depending on the `action` parameter.
#[derive(Serialize)]
#[serde(untagged)]
pub enum JukeboxResponseData {
    Status {
        #[serde(rename = "jukeboxStatus")]
        jukebox_status: JukeboxStatus,
    },
    Playlist {
        #[serde(rename = "jukeboxPlaylist")]
        jukebox_playlist: JukeboxPlaylist,
    },
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct JukeboxControlParams {
    /// Valid actions: get, status, set, start, stop, skip, add, clear,
    /// remove, shuffle, setGain.
    pub action: String,
    pub index: Option<i32>,
    pub offset: Option<i32>,
    #[serde(default)]
    pub id: Vec<String>,
    pub gain: Option<f32>,
}

/// GET/POST /rest/jukeboxControl
pub async fn jukebox_control(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<JukeboxControlParams>,
) -> SubsonicResponse<JukeboxResponseData> {
    todo!()
}
