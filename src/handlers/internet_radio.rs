use axum::extract::State;
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use diesel_async::RunQueryDsl;
use opensubsonic::data::InternetRadioStation;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::get_conn;
use crate::response::{Empty, SubsonicResponse};
use crate::state::AppState;

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
// DB row
// ---------------------------------------------------------------------------

#[derive(diesel::QueryableByName)]
struct RadioRow {
    #[diesel(sql_type = Integer)]
    pub db_id: i32,
    #[diesel(sql_type = Text)]
    pub upstream_id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub stream_url: String,
    #[diesel(sql_type = Text)]
    pub metadata_json: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET/POST /rest/getInternetRadioStations — no extra parameters
pub async fn get_internet_radio_stations(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
) -> Result<SubsonicResponse<InternetRadioStationsResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;

    let rows: Vec<RadioRow> = sql_query(
        "SELECT irs.id AS db_id, irs.upstream_id, irs.name, irs.stream_url, \
         irs.metadata_json \
         FROM internet_radio_stations irs \
         ORDER BY irs.name COLLATE NOCASE",
    )
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let stations: Vec<InternetRadioStation> = rows
        .into_iter()
        .map(|row| {
            // Try to deserialize stored metadata; fall back to a minimal station.
            serde_json::from_str::<InternetRadioStation>(&row.metadata_json)
                .unwrap_or_else(|_| InternetRadioStation {
                    id: row.upstream_id.clone(),
                    name: row.name,
                    stream_url: row.stream_url,
                    home_page_url: None,
                })
        })
        .collect();

    Ok(InternetRadioStationsResponse {
        internet_radio_stations: InternetRadioStationsBody {
            internet_radio_station: stations,
        },
    }
    .into())
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Internet radio management is not supported"))
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Internet radio management is not supported"))
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Internet radio management is not supported"))
}
