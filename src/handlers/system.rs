use opensubsonic::data::{License, OpenSubsonicExtension, TokenInfo};
use serde::Serialize;

use crate::auth::SubsonicAuth;
use crate::response::{Empty, SubsonicResponse};

#[derive(Serialize)]
pub struct LicenseResponse {
    pub license: License,
}

#[derive(Serialize)]
pub struct OpenSubsonicExtensionsResponse {
    #[serde(rename = "openSubsonicExtensions")]
    pub open_subsonic_extensions: Vec<OpenSubsonicExtension>,
}

#[derive(Serialize)]
pub struct TokenInfoResponse {
    #[serde(rename = "tokenInfo")]
    pub token_info: TokenInfo,
}

// All system endpoints take no parameters beyond auth.

/// GET/POST /rest/ping
pub async fn ping(_auth: SubsonicAuth) -> SubsonicResponse<Empty> {
    todo!()
}

/// GET/POST /rest/getLicense
pub async fn get_license(_auth: SubsonicAuth) -> SubsonicResponse<LicenseResponse> {
    todo!()
}

/// GET/POST /rest/getOpenSubsonicExtensions
pub async fn get_open_subsonic_extensions(
    _auth: SubsonicAuth,
) -> SubsonicResponse<OpenSubsonicExtensionsResponse> {
    todo!()
}

/// GET/POST /rest/tokenInfo
pub async fn token_info(_auth: SubsonicAuth) -> SubsonicResponse<TokenInfoResponse> {
    todo!()
}
