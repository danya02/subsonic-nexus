use opensubsonic::data::InternetRadioStation;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct InternetRadioStationsBody {
    #[serde(rename = "internetRadioStation")]
    pub internet_radio_station: Vec<InternetRadioStation>,
}

#[derive(Serialize)]
pub struct InternetRadioStationsResponse {
    #[serde(rename = "internetRadioStations")]
    pub internet_radio_stations: InternetRadioStationsBody,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET/POST /rest/getInternetRadioStations — no extra parameters
pub async fn get_internet_radio_stations(
    _auth: SubsonicAuth,
) -> SubsonicResponse<InternetRadioStationsResponse> {
    todo!()
}

// --- createInternetRadioStation ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInternetRadioStationParams {
    pub stream_url: String,
    pub name: String,
    pub homepage_url: Option<String>,
}

/// GET/POST /rest/createInternetRadioStation
pub async fn create_internet_radio_station(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<CreateInternetRadioStationParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- updateInternetRadioStation ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInternetRadioStationParams {
    pub id: String,
    pub stream_url: String,
    pub name: String,
    pub homepage_url: Option<String>,
}

/// GET/POST /rest/updateInternetRadioStation
pub async fn update_internet_radio_station(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<UpdateInternetRadioStationParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- deleteInternetRadioStation ---

#[derive(Deserialize)]
pub struct DeleteInternetRadioStationParams {
    pub id: String,
}

/// GET/POST /rest/deleteInternetRadioStation
pub async fn delete_internet_radio_station(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeleteInternetRadioStationParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}
