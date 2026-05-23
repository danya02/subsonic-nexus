use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getInternetRadioStations
pub async fn get_internet_radio_stations(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/createInternetRadioStation
pub async fn create_internet_radio_station(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/updateInternetRadioStation
pub async fn update_internet_radio_station(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deleteInternetRadioStation
pub async fn delete_internet_radio_station(_auth: SubsonicAuth) -> Response {
    todo!()
}
