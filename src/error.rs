use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::response::{API_VERSION, SERVER_TYPE, SubsonicResponse, SubsonicResponseBody};

/// Subsonic API error codes as defined by the OpenSubsonic spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    Generic = 0,
    MissingParameter = 10,
    ClientMustUpgrade = 20,
    ServerMustUpgrade = 30,
    WrongCredentials = 40,
    TokenAuthNotSupported = 41,
    AuthMechanismNotSupported = 42,
    ConflictingAuthentication = 43,
    InvalidApiKey = 44,
    NotAuthorized = 50,
    TrialExpired = 60,
    NotFound = 70,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

/// A Subsonic API error response, ready to be returned from Axum handlers.
pub struct SubsonicError {
    pub code: ErrorCode,
    pub message: String,
}

impl SubsonicError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn not_authorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotAuthorized, message)
    }

    pub fn generic(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Generic, message)
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: ApiError,
}

impl IntoResponse for SubsonicError {
    fn into_response(self) -> Response {
        let body = SubsonicResponse {
            response: SubsonicResponseBody {
                status: "failed",
                version: API_VERSION,
                server_type: SERVER_TYPE,
                open_subsonic: true,
                data: ErrorBody {
                    error: ApiError {
                        code: self.code as i32,
                        message: self.message,
                    },
                },
            },
        };
        body.into_response()
    }
}
