use axum::{
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
};
use serde::Deserialize;

/// Parsed subsonic authentication parameters from query string or form body.
/// At this scaffolding stage the extractor always succeeds — validation is a no-op.
#[derive(Debug, Clone)]
pub struct SubsonicAuth {
    pub username: Option<String>,
    pub credential: Credential,
    pub client: String,
    pub version: String,
    pub format: Format,
}

#[derive(Debug, Clone)]
pub enum Credential {
    /// Legacy plain-text password (`p=` param, hex-encoded).
    Plain(String),
    /// Token-based auth (`t=` MD5 token + `s=` salt, API ≥ 1.13.0).
    Token { token: String, salt: String },
    /// OpenSubsonic API key (`apiKey=` param, no username required).
    ApiKey(String),
    /// No credential provided (stub / unauthenticated).
    None,
}

#[derive(Debug, Clone, Default)]
pub enum Format {
    #[default]
    Json,
    Xml,
}

/// Raw query-string parameters we care about for auth.
#[derive(Debug, Deserialize, Default)]
struct RawAuthParams {
    u: Option<String>,
    p: Option<String>,
    t: Option<String>,
    s: Option<String>,
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
    c: Option<String>,
    v: Option<String>,
    f: Option<String>,
}

impl RawAuthParams {
    fn into_auth(self) -> SubsonicAuth {
        let credential = if let Some(key) = self.api_key {
            Credential::ApiKey(key)
        } else if let (Some(token), Some(salt)) = (self.t, self.s) {
            Credential::Token { token, salt }
        } else if let Some(plain) = self.p {
            Credential::Plain(plain)
        } else {
            Credential::None
        };

        let auth_type = match &credential {
            Credential::ApiKey(_) => "api_key",
            Credential::Token { .. } => "token",
            Credential::Plain(_) => "plain",
            Credential::None => "none",
        };

        if matches!(credential, Credential::None) && self.u.is_some() {
            tracing::warn!(username = ?self.u, "auth: username present but no credential provided");
        }

        let format = match self.f.as_deref() {
            Some("xml") => Format::Xml,
            _ => Format::Json,
        };

        let auth = SubsonicAuth {
            username: self.u,
            credential,
            client: self.c.unwrap_or_default(),
            version: self.v.unwrap_or_else(|| "1.16.1".to_string()),
            format,
        };
        tracing::debug!(username = ?auth.username, client = %auth.client, version = %auth.version, auth_type, "auth extracted");
        auth
    }
}

impl<S: Send + Sync> FromRequestParts<S> for SubsonicAuth {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract auth params from the query string via Axum's Query extractor.
        // For POST form-body endpoints the same params are typically present in the query string
        // as well (most clients include them there). Full form-body support can be
        // added when real authentication is implemented.
        let Query(params) = Query::<RawAuthParams>::from_request_parts(parts, state)
            .await
            .unwrap_or(Query(RawAuthParams::default()));
        Ok(params.into_auth())
    }
}
