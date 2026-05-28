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
    Empty {}.into()
}

/// GET/POST /rest/getLicense
pub async fn get_license(_auth: SubsonicAuth) -> SubsonicResponse<LicenseResponse> {
    LicenseResponse {
        license: License {
            valid: true,
            email: None,
            license_expires: None,
            trial_expires: None,
        },
    }
    .into()
}

/// GET/POST /rest/getOpenSubsonicExtensions
pub async fn get_open_subsonic_extensions() -> SubsonicResponse<OpenSubsonicExtensionsResponse> {
    OpenSubsonicExtensionsResponse {
        open_subsonic_extensions: vec![OpenSubsonicExtension {
            name: "formPost".to_string(),
            versions: vec![1],
        }],
    }
    .into()
}

/// GET/POST /rest/tokenInfo
pub async fn token_info(auth: SubsonicAuth) -> SubsonicResponse<TokenInfoResponse> {
    TokenInfoResponse {
        token_info: TokenInfo {
            username: auth.username.unwrap_or_else(|| "anonymous".to_string()),
        },
    }
    .into()
}
