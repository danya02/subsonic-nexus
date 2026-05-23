//! Admin / management UI handlers.
//!
//! Routes:
//!   GET  /          — HTML dashboard: server info, row counts, scan button
//!   POST /admin/scan — trigger an immediate scan of all configured backends

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use diesel::prelude::*;
use serde_json::json;

use crate::db::schema::*;
use crate::state::AppState;

// ── GET / ─────────────────────────────────────────────────────────────────────

pub async fn index(State(state): State<AppState>) -> Response {
    match build_index_html(&state) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build dashboard: {e}"),
        )
            .into_response(),
    }
}

fn build_index_html(state: &AppState) -> Result<String, Box<dyn std::error::Error>> {
    let mut conn = state.pool.get()?;

    // ── Row counts (aggregate across all servers) ──────────────────────────
    let total_artists: i64 = artists::table.count().get_result(&mut conn)?;
    let unique_artist_keys: i64 = artists::table
        .select(diesel::dsl::count(artists::aggregation_key).aggregate_distinct())
        .first(&mut conn)?;

    let total_albums: i64 = albums::table.count().get_result(&mut conn)?;
    let unique_album_keys: i64 = albums::table
        .select(diesel::dsl::count(albums::aggregation_key).aggregate_distinct())
        .first(&mut conn)?;

    let total_songs: i64 = songs::table.count().get_result(&mut conn)?;
    let total_podcasts: i64 = podcast_channels::table.count().get_result(&mut conn)?;
    let total_episodes: i64 = podcast_episodes::table.count().get_result(&mut conn)?;
    let total_radio: i64 = internet_radio_stations::table.count().get_result(&mut conn)?;

    // ── Per-server info from the DB ────────────────────────────────────────
    let db_servers: Vec<(i32, String, String, i32, Option<String>)> =
        upstream_servers::table
            .select((
                upstream_servers::id,
                upstream_servers::name,
                upstream_servers::url,
                upstream_servers::priority,
                upstream_servers::last_scanned_at,
            ))
            .order(upstream_servers::priority.asc())
            .load(&mut conn)?;

    // Per-server row counts
    let server_artist_counts: Vec<(i32, i64)> = artists::table
        .group_by(artists::server_id)
        .select((artists::server_id, diesel::dsl::count_star()))
        .load(&mut conn)?;

    let server_album_counts: Vec<(i32, i64)> = albums::table
        .group_by(albums::server_id)
        .select((albums::server_id, diesel::dsl::count_star()))
        .load(&mut conn)?;

    let server_song_counts: Vec<(i32, i64)> = songs::table
        .group_by(songs::server_id)
        .select((songs::server_id, diesel::dsl::count_star()))
        .load(&mut conn)?;

    let lookup = |v: &[(i32, i64)], id: i32| v.iter().find(|(s, _)| *s == id).map_or(0, |(_, n)| *n);

    // ── Servers configured in nexus.toml (may not be scanned yet) ─────────
    let config_servers: Vec<&str> = state.config.servers.iter().map(|s| s.name.as_str()).collect();

    // ── Build HTML ─────────────────────────────────────────────────────────
    let server_rows: String = db_servers
        .iter()
        .map(|(id, name, url, priority, last_scanned)| {
            let artists_n = lookup(&server_artist_counts, *id);
            let albums_n = lookup(&server_album_counts, *id);
            let songs_n = lookup(&server_song_counts, *id);
            let scanned = last_scanned.as_deref().unwrap_or("never");
            let in_config = if config_servers.contains(&name.as_str()) {
                r#"<span class="badge ok">✓ configured</span>"#
            } else {
                r#"<span class="badge warn">⚠ not in nexus.toml</span>"#
            };
            format!(
                r#"<tr>
  <td><strong>{name}</strong><br><small class="muted">{url}</small></td>
  <td class="center">{priority}</td>
  <td class="center">{artists_n}</td>
  <td class="center">{albums_n}</td>
  <td class="center">{songs_n}</td>
  <td class="center"><code>{scanned}</code></td>
  <td class="center">{in_config}</td>
</tr>"#
            )
        })
        .collect();

    // Servers in config but not yet in DB
    let unscanned_rows: String = state.config.servers.iter()
        .filter(|s| !db_servers.iter().any(|(_, name, _, _, _)| name == &s.name))
        .map(|s| format!(
            r#"<tr class="dim">
  <td><strong>{}</strong><br><small class="muted">{}</small></td>
  <td class="center">{}</td>
  <td class="center">—</td>
  <td class="center">—</td>
  <td class="center">—</td>
  <td class="center"><code>not scanned</code></td>
  <td class="center"><span class="badge warn">⚠ pending scan</span></td>
</tr>"#,
            s.name, s.url, s.priority
        ))
        .collect();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>subsonic-nexus</title>
  <style>
    :root {{
      --bg: #0f1117; --surface: #1a1d27; --border: #2e3149;
      --text: #e2e8f0; --muted: #8892a4; --accent: #6366f1;
      --ok: #22c55e; --warn: #f59e0b; --err: #ef4444;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{ background: var(--bg); color: var(--text); font-family: system-ui, sans-serif; font-size: 14px; padding: 2rem; }}
    h1 {{ font-size: 1.6rem; font-weight: 700; color: var(--accent); margin-bottom: 0.2rem; }}
    h2 {{ font-size: 1rem; font-weight: 600; text-transform: uppercase; letter-spacing: .08em; color: var(--muted); margin: 2rem 0 0.75rem; }}
    .subtitle {{ color: var(--muted); margin-bottom: 2rem; }}
    .stats {{ display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 0.5rem; }}
    .stat {{ background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 1rem 1.5rem; min-width: 120px; }}
    .stat .n {{ font-size: 2rem; font-weight: 700; color: var(--accent); line-height: 1; }}
    .stat .label {{ color: var(--muted); font-size: 0.8rem; margin-top: 0.3rem; }}
    .stat .sub {{ color: var(--muted); font-size: 0.75rem; margin-top: 0.1rem; }}
    table {{ width: 100%; border-collapse: collapse; background: var(--surface); border-radius: 8px; overflow: hidden; border: 1px solid var(--border); }}
    th {{ background: #12151f; color: var(--muted); font-weight: 600; text-transform: uppercase; font-size: 0.75rem; letter-spacing: .06em; padding: 0.6rem 1rem; text-align: left; }}
    td {{ padding: 0.75rem 1rem; border-top: 1px solid var(--border); vertical-align: top; }}
    .center {{ text-align: center; }}
    .muted {{ color: var(--muted); }}
    tr.dim td {{ color: var(--muted); }}
    code {{ background: #12151f; padding: 0.1rem 0.4rem; border-radius: 4px; font-size: 0.8rem; }}
    .badge {{ display: inline-block; padding: 0.15rem 0.5rem; border-radius: 4px; font-size: 0.75rem; font-weight: 600; }}
    .badge.ok {{ background: #14532d; color: var(--ok); }}
    .badge.warn {{ background: #451a03; color: var(--warn); }}
    .scan-bar {{ display: flex; gap: 1rem; align-items: center; margin: 2rem 0 1rem; }}
    button {{ background: var(--accent); color: #fff; border: none; border-radius: 6px; padding: 0.6rem 1.4rem; font-size: 0.9rem; font-weight: 600; cursor: pointer; transition: opacity .15s; }}
    button:hover {{ opacity: 0.85; }}
    button:active {{ opacity: 0.7; }}
    #scan-status {{ color: var(--muted); font-size: 0.85rem; }}
  </style>
</head>
<body>
  <h1>subsonic-nexus</h1>
  <p class="subtitle">OpenSubsonic aggregation proxy — <a href="/rest/ping" style="color:var(--accent)">/rest/ping</a></p>

  <h2>Library totals</h2>
  <div class="stats">
    <div class="stat">
      <div class="n">{unique_artist_keys}</div>
      <div class="label">Merged artists</div>
      <div class="sub">{total_artists} source rows</div>
    </div>
    <div class="stat">
      <div class="n">{unique_album_keys}</div>
      <div class="label">Merged albums</div>
      <div class="sub">{total_albums} source rows</div>
    </div>
    <div class="stat">
      <div class="n">{total_songs}</div>
      <div class="label">Songs</div>
    </div>
    <div class="stat">
      <div class="n">{total_podcasts}</div>
      <div class="label">Podcast channels</div>
      <div class="sub">{total_episodes} episodes</div>
    </div>
    <div class="stat">
      <div class="n">{total_radio}</div>
      <div class="label">Radio stations</div>
    </div>
  </div>

  <div class="scan-bar">
    <button onclick="triggerScan()">⟳ Scan all backends now</button>
    <span id="scan-status"></span>
  </div>

  <h2>Backends</h2>
  <table>
    <thead><tr>
      <th>Server</th><th>Priority</th><th>Artists</th><th>Albums</th>
      <th>Songs</th><th>Last scanned</th><th>Status</th>
    </tr></thead>
    <tbody>{server_rows}{unscanned_rows}</tbody>
  </table>

  <script>
    async function triggerScan() {{
      const btn = document.querySelector('button');
      const status = document.getElementById('scan-status');
      btn.disabled = true;
      status.textContent = 'Starting scan…';
      try {{
        const r = await fetch('/admin/scan', {{ method: 'POST' }});
        const j = await r.json();
        status.textContent = j.message ?? JSON.stringify(j);
        // Reload stats after a short delay
        setTimeout(() => window.location.reload(), 2000);
      }} catch(e) {{
        status.textContent = 'Error: ' + e;
      }} finally {{
        btn.disabled = false;
      }}
    }}
  </script>
</body>
</html>"#
    );

    Ok(html)
}

// ── POST /admin/scan ──────────────────────────────────────────────────────────

pub async fn trigger_scan(State(state): State<AppState>) -> Response {
    let server_count = state.config.servers.len();

    if server_count == 0 {
        return (
            StatusCode::OK,
            axum::Json(json!({
                "status": "noop",
                "message": "No servers configured in nexus.toml"
            })),
        )
            .into_response();
    }

    // Spawn the scan as a background task so we can return immediately.
    tokio::spawn(async move {
        for cfg in &state.config.servers {
            let mut conn = match state.pool.get() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[scan] Failed to get DB connection for {}: {e}", cfg.name);
                    continue;
                }
            };

            let server_id = match crate::scanner::ensure_server_row(&mut conn, cfg) {
                Ok((id, _)) => id,
                Err(e) => {
                    eprintln!("[scan] Failed to ensure server row for {}: {e}", cfg.name);
                    continue;
                }
            };

            eprintln!("[scan] Starting scan of '{}' …", cfg.name);
            match crate::scanner::scan_server(&mut conn, cfg, server_id).await {
                Ok(stats) => eprintln!(
                    "[scan] '{}' done — {} artists, {} albums, {} songs, \
                     {} podcast channels, {} episodes, {} radio stations",
                    cfg.name,
                    stats.artists,
                    stats.albums,
                    stats.songs,
                    stats.podcast_channels,
                    stats.podcast_episodes,
                    stats.radio_stations,
                ),
                Err(e) => eprintln!("[scan] '{}' failed: {e}", cfg.name),
            }
        }
    });

    (
        StatusCode::ACCEPTED,
        axum::Json(json!({
            "status": "started",
            "message": format!("Scan started for {server_count} server(s). Check the dashboard for results.")
        })),
    )
        .into_response()
}
