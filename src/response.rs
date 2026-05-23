use axum::{Json, response::{IntoResponse, Response}};
use serde::Serialize;

pub const API_VERSION: &str = "1.16.1";
pub const SERVER_TYPE: &str = "subsonic-nexus";

#[derive(Serialize)]
pub struct SubsonicResponse<T: Serialize> {
    #[serde(rename = "subsonic-response")]
    pub response: SubsonicResponseBody<T>,
}

#[derive(Serialize)]
pub struct SubsonicResponseBody<T: Serialize> {
    pub status: &'static str,
    pub version: &'static str,
    #[serde(rename = "type")]
    pub server_type: &'static str,
    #[serde(rename = "openSubsonic")]
    pub open_subsonic: bool,
    #[serde(flatten)]
    pub data: T,
}

impl<T: Serialize> SubsonicResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            response: SubsonicResponseBody {
                status: "ok",
                version: API_VERSION,
                server_type: SERVER_TYPE,
                open_subsonic: true,
                data,
            },
        }
    }
}

impl<T: Serialize + Send> IntoResponse for SubsonicResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
