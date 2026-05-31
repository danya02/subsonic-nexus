use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::Deserialize;

use crate::config::AuthConfig;
use crate::error::{ErrorCode, SubsonicError};
use crate::state::AppState;

/// Parsed and validated Subsonic authentication parameters.
///
/// If `[nexus.auth]` is configured, the extractor rejects requests with wrong
/// credentials before they reach any handler.  If no auth is configured, all
/// logins are accepted (no-auth mode).
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
    /// Legacy plain-text password (`p=` param, optionally hex-encoded with `enc:` prefix).
    Plain(String),
    /// Token-based auth (`t=` MD5 token + `s=` salt, API ≥ 1.13.0).
    Token { token: String, salt: String },
    /// OpenSubsonic API key (`apiKey=` param, no username required).
    ApiKey(String),
    /// No credential provided.
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

impl SubsonicAuth {
    fn validate(&self, cfg: &AuthConfig) -> Result<(), SubsonicError> {
        let wrong = || SubsonicError::new(ErrorCode::WrongCredentials, "Wrong username or password.");

        match &self.credential {
            Credential::ApiKey(key) => {
                if cfg.api_keys.iter().any(|k| k == key) {
                    Ok(())
                } else {
                    Err(SubsonicError::new(ErrorCode::InvalidApiKey, "Invalid API key."))
                }
            }
            Credential::Token { token, salt } => {
                let username = self.username.as_deref().unwrap_or("");
                if username != cfg.username {
                    return Err(wrong());
                }
                let expected = format!("{:x}", md5::compute(format!("{}{}", cfg.password, salt)));
                if token == &expected {
                    Ok(())
                } else {
                    Err(wrong())
                }
            }
            Credential::Plain(p) => {
                let username = self.username.as_deref().unwrap_or("");
                if username != cfg.username {
                    return Err(wrong());
                }
                let password = decode_plain_password(p);
                if password == cfg.password {
                    Ok(())
                } else {
                    Err(wrong())
                }
            }
            Credential::None => Err(wrong()),
        }
    }
}

/// Decodes the `p=` parameter: strips `enc:` prefix and hex-decodes if present,
/// otherwise returns the value as-is.
fn decode_plain_password(p: &str) -> String {
    if let Some(hex) = p.strip_prefix("enc:") {
        (0..hex.len())
            .step_by(2)
            .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
            .collect::<Vec<u8>>()
            .pipe(|bytes| String::from_utf8(bytes).unwrap_or_default())
    } else {
        p.to_owned()
    }
}

// Tiny helper to avoid a local variable just for the pipe call.
trait Pipe: Sized {
    fn pipe<F: FnOnce(Self) -> R, R>(self, f: F) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

impl FromRequestParts<AppState> for SubsonicAuth {
    type Rejection = SubsonicError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let Query(params) = Query::<RawAuthParams>::from_request_parts(parts, state)
            .await
            .unwrap_or(Query(RawAuthParams::default()));
        let auth = params.into_auth();

        if let Some(auth_cfg) = &state.config.nexus.auth {
            auth.validate(auth_cfg)?;
        }

        Ok(auth)
    }
}
