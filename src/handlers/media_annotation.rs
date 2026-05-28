use std::time::Duration;

use axum::extract::State;
use serde::Deserialize;

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::nexus::{
    IdTemplate, build_upstream_url, get_conn, parse_entity_id, query_song_by_nexus_id,
};
use crate::response::{Empty, SubsonicResponse};
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Shared helper: fire-and-forget POST to an upstream REST endpoint
// ---------------------------------------------------------------------------

/// Proxy a write operation to an upstream server.
///
/// Builds the authenticated URL, sends the request, and logs the outcome.
/// Uses a 15-second connect+read timeout so slow upstreams don't block
/// handlers indefinitely.
async fn proxy_write(
    server_cfg: &crate::config::ServerConfig,
    endpoint: &str,
    params: &[(&str, &str)],
) {
    let url = build_upstream_url(server_cfg, endpoint, params);
    tracing::info!(endpoint, server = %server_cfg.name, "write proxy: sending");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    match client.get(&url).send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::info!(endpoint, server = %server_cfg.name, %status, "write proxy: ok");
            tracing::debug!(endpoint, server = %server_cfg.name, body = &body[..body.len().min(200)], "write proxy: response body");
        }
        Err(e) => {
            tracing::warn!(endpoint, server = %server_cfg.name, error = %e, "write proxy: failed");
        }
    }
}

// ---------------------------------------------------------------------------
// star / unstar
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarParams {
    #[serde(default)]
    pub id: Vec<String>,
    #[serde(default)]
    pub album_id: Vec<String>,
    #[serde(default)]
    pub artist_id: Vec<String>,
}

/// GET/POST /rest/star
pub async fn star(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<StarParams>,
) -> SubsonicResponse<Empty> {
    proxy_annotation(&state, "star", params).await;
    Empty {}.into()
}

/// GET/POST /rest/unstar
pub async fn unstar(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<StarParams>,
) -> SubsonicResponse<Empty> {
    proxy_annotation(&state, "unstar", params).await;
    Empty {}.into()
}

/// Proxy a star/unstar call to the `write_target` server (if configured),
/// translating nexus IDs to upstream IDs.
async fn proxy_annotation(state: &AppState, endpoint: &str, params: StarParams) {
    let Some(ref target_name) = state.config.nexus.write_target else {
        tracing::debug!(
            endpoint,
            "proxy_annotation: no write_target configured, skipping"
        );
        return;
    };
    let Some(server_cfg) = state.config.server_by_name(target_name) else {
        tracing::warn!(endpoint, write_target = %target_name, "proxy_annotation: write_target server not found in config");
        return;
    };
    tracing::debug!(endpoint, write_target = %target_name, song_ids = params.id.len(), album_ids = params.album_id.len(), artist_ids = params.artist_id.len(), "proxy_annotation");

    let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);
    let mut query_params: Vec<String> = Vec::new();

    for nexus_id in &params.id {
        let parsed = parse_entity_id(template, nexus_id);
        query_params.push(format!("id={}", parsed.upstream_id));
    }
    for nexus_id in &params.album_id {
        let parsed = parse_entity_id(template, nexus_id);
        query_params.push(format!("albumId={}", parsed.upstream_id));
    }
    for nexus_id in &params.artist_id {
        let parsed = parse_entity_id(template, nexus_id);
        query_params.push(format!("artistId={}", parsed.upstream_id));
    }

    if query_params.is_empty() {
        tracing::debug!(endpoint, "proxy_annotation: no IDs to proxy, skipping");
        return;
    }

    // build_upstream_url uses &[(&str, &str)] — build owned pairs
    let pairs: Vec<(String, String)> = query_params
        .iter()
        .filter_map(|s| {
            let mut it = s.splitn(2, '=');
            Some((it.next()?.to_owned(), it.next()?.to_owned()))
        })
        .collect();
    let ref_pairs: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    proxy_write(server_cfg, endpoint, &ref_pairs).await;
}

// ---------------------------------------------------------------------------
// setRating
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SetRatingParams {
    pub id: String,
    /// 0 to remove rating; 1–5 to set it.
    pub rating: i32,
}

/// GET/POST /rest/setRating
pub async fn set_rating(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<SetRatingParams>,
) -> SubsonicResponse<Empty> {
    tracing::debug!(song_id = %params.id, rating = params.rating, "setRating");
    if let Some(ref target_name) = state.config.nexus.write_target
        && let Some(server_cfg) = state.config.server_by_name(target_name) {
            let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);
            let parsed = parse_entity_id(template, &params.id);
            let rating_s = params.rating.to_string();
            proxy_write(
                server_cfg,
                "setRating",
                &[("id", &parsed.upstream_id), ("rating", &rating_s)],
            )
            .await;
        }
    Empty {}.into()
}

// ---------------------------------------------------------------------------
// scrobble
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ScrobbleParams {
    pub id: String,
    pub time: Option<i64>,
    pub submission: Option<bool>,
}

/// GET/POST /rest/scrobble
///
/// Proxied to the song's own upstream server (the one it was imported from),
/// not the `write_target`.  This records the play on the correct server so
/// its play count / Last.fm scrobble reflects reality.
pub async fn scrobble(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<ScrobbleParams>,
) -> SubsonicResponse<Empty> {
    tracing::debug!(song_id = %params.id, submission = ?params.submission, "scrobble");
    // Look up song to find its canonical server.
    if let Ok(mut conn) = get_conn(&state.pool).await {
        let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);
        if let Ok(Some(song)) = query_song_by_nexus_id(&mut conn, template, &params.id).await
            && let Some(server_cfg) = state.config.server_by_name(&song.server_name) {
                let time_s;
                let submission_s;
                let mut extra: Vec<(&str, &str)> = vec![("id", &song.upstream_id)];
                if let Some(t) = params.time {
                    time_s = t.to_string();
                    extra.push(("time", &time_s));
                }
                if let Some(s) = params.submission {
                    submission_s = s.to_string();
                    extra.push(("submission", &submission_s));
                }
                proxy_write(server_cfg, "scrobble", &extra).await;
            }
    }
    Empty {}.into()
}

// ---------------------------------------------------------------------------
// reportPlayback
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPlaybackParams {
    pub media_id: String,
    pub media_type: String,
    pub position_ms: i64,
    pub state: String,
    pub ignore_scrobble: Option<bool>,
    pub playback_rate: Option<f32>,
}

/// GET/POST /rest/reportPlayback
///
/// OpenSubsonic extension.  Like scrobble, proxied to the song's own server.
pub async fn report_playback(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<ReportPlaybackParams>,
) -> SubsonicResponse<Empty> {
    tracing::debug!(media_id = %params.media_id, state = %params.state, position_ms = params.position_ms, "reportPlayback");
    if let Ok(mut conn) = get_conn(&state.pool).await {
        let template = IdTemplate::from_config(&state.config.nexus.entity_id_template);
        if let Ok(Some(song)) = query_song_by_nexus_id(&mut conn, template, &params.media_id).await
            && let Some(server_cfg) = state.config.server_by_name(&song.server_name) {
                let position_s = params.position_ms.to_string();
                let ignore_s;
                let rate_s;
                let mut extra: Vec<(&str, &str)> = vec![
                    ("mediaId", &song.upstream_id),
                    ("mediaType", &params.media_type),
                    ("positionMs", &position_s),
                    ("state", &params.state),
                ];
                if let Some(ig) = params.ignore_scrobble {
                    ignore_s = ig.to_string();
                    extra.push(("ignoreScrobble", &ignore_s));
                }
                if let Some(r) = params.playback_rate {
                    rate_s = r.to_string();
                    extra.push(("playbackRate", &rate_s));
                }
                proxy_write(server_cfg, "reportPlayback", &extra).await;
            }
    }
    Empty {}.into()
}
