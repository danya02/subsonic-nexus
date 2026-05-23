use axum::{
    extract::{FromRequest, Request},
    http::{StatusCode, header::CONTENT_TYPE},
};
use serde::de::DeserializeOwned;

/// Axum extractor that reads typed endpoint parameters from either the query
/// string (GET) or the URL-encoded form body (POST), merging both when present.
///
/// The OpenSubsonic spec defines every endpoint as accepting both GET (params
/// in the query string) and POST (params in `application/x-www-form-urlencoded`
/// body). Some clients include params in the query string even for POST
/// requests, so we always parse both sources and concatenate them.
///
/// Auth params (`u`, `p`, `t`, `s`, `apiKey`, `c`, `v`, `f`) are present in
/// the combined string too but are ignored here because the endpoint-specific
/// params struct won't declare those fields, and serde silently skips unknown
/// keys by default.
pub struct QueryOrForm<T>(pub T);

impl<S, T> FromRequest<S> for QueryOrForm<T>
where
    T: DeserializeOwned + Send + 'static,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        // Always read the query string.
        let query = parts.uri.query().unwrap_or("").to_owned();

        // Read the form body only when the content-type says so.
        let is_form_body = parts
            .headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.starts_with("application/x-www-form-urlencoded"));

        let body_str = if is_form_body {
            let bytes = axum::body::to_bytes(body, 2 * 1024 * 1024)
                .await
                .map_err(|e| (StatusCode::PAYLOAD_TOO_LARGE, e.to_string()))?;
            String::from_utf8_lossy(&bytes).into_owned()
        } else {
            String::new()
        };

        // Concatenate — both sides use the same `key=value&…` encoding.
        let combined = match (query.is_empty(), body_str.is_empty()) {
            (true, _) => body_str,
            (_, true) => query,
            _ => format!("{}&{}", query, body_str),
        };

        let params = serde_urlencoded::from_str::<T>(&combined)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

        Ok(QueryOrForm(params))
    }
}
