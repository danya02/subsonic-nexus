//! Upstream server scanner.
//!
//! `scan_server` fetches the full library from one upstream OpenSubsonic server
//! and upserts it into the local database.  The aggregation key for each entity
//! is computed via the per-server templates defined in `nexus.toml`.
//!
//! # Rescan detection
//! The templates used during the previous scan are stored in the
//! `upstream_servers` row.  On startup, `ensure_server_row` compares them to
//! the current config.  If they differ, all rows for that server are deleted
//! and a fresh scan is scheduled.

pub mod template;

#[cfg(test)]
mod tests;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opensubsonic::{Auth, Client};

use crate::config::ServerConfig;
use crate::db::models::*;
use crate::db::schema::*;
use crate::db::{AsyncSqliteConnection, DbPool};
use template::{Template, eval_album, eval_artist};

// ── Entry point ───────────────────────────────────────────────────────────────

/// Ensure the `upstream_servers` row for `cfg` exists and has up-to-date
/// templates.  Returns the local server id and a flag indicating whether a
/// full rescan is needed (templates changed or first run).
pub async fn ensure_server_row(
    conn: &mut AsyncSqliteConnection,
    cfg: &ServerConfig,
) -> QueryResult<(i32, bool)> {
    use upstream_servers::dsl as s;

    let artist_tmpl = &cfg.matching.artist;
    let album_tmpl = &cfg.matching.album;

    // Look for an existing row by (name, url) — the stable identity of a server.
    let existing: Option<UpstreamServer> = s::upstream_servers
        .filter(s::name.eq(&cfg.name).and(s::url.eq(&cfg.url)))
        .first(conn)
        .await
        .optional()?;

    match existing {
        Some(row) => {
            let templates_changed = row.artist_key_template.as_deref()
                != Some(artist_tmpl.as_str())
                || row.album_key_template.as_deref() != Some(album_tmpl.as_str());

            if templates_changed {
                tracing::debug!(server = %cfg.name, server_id = row.id, "ensure_server_row: templates changed, wiping stale data");
                // Wipe all data for this server — aggregation keys are stale.
                delete_server_data(conn, row.id).await?;

                diesel::update(s::upstream_servers.find(row.id))
                    .set((
                        s::artist_key_template.eq(artist_tmpl),
                        s::album_key_template.eq(album_tmpl),
                        s::priority.eq(cfg.priority),
                        s::last_scanned_at.eq::<Option<&str>>(None),
                    ))
                    .execute(conn)
                    .await?;
            } else {
                tracing::debug!(server = %cfg.name, server_id = row.id, "ensure_server_row: existing server, templates unchanged");
                // Just refresh priority in case it changed in config.
                diesel::update(s::upstream_servers.find(row.id))
                    .set(s::priority.eq(cfg.priority))
                    .execute(conn)
                    .await?;
            }

            Ok((row.id, templates_changed))
        }
        None => {
            tracing::debug!(server = %cfg.name, "ensure_server_row: first time seeing this server");
            // First time we see this server.
            diesel::insert_into(s::upstream_servers)
                .values(NewUpstreamServer {
                    name: &cfg.name,
                    url: &cfg.url,
                    priority: cfg.priority,
                    last_scanned_at: None,
                    artist_key_template: Some(artist_tmpl),
                    album_key_template: Some(album_tmpl),
                })
                .execute(conn)
                .await?;

            let id: i32 = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
                "last_insert_rowid()",
            ))
            .get_result(conn)
            .await?;

            Ok((id, true))
        }
    }
}

/// Delete all library data for a server (artists, albums, songs, podcasts,
/// radio stations).  The `upstream_servers` row itself is kept.
pub async fn delete_server_data(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
) -> QueryResult<()> {
    // ON DELETE CASCADE handles the child tables, but let's be explicit.
    let songs_deleted = diesel::delete(songs::table.filter(songs::server_id.eq(server_id)))
        .execute(conn)
        .await?;
    let albums_deleted = diesel::delete(albums::table.filter(albums::server_id.eq(server_id)))
        .execute(conn)
        .await?;
    let artists_deleted = diesel::delete(artists::table.filter(artists::server_id.eq(server_id)))
        .execute(conn)
        .await?;
    let podcast_episodes_deleted =
        diesel::delete(podcast_episodes::table.filter(podcast_episodes::server_id.eq(server_id)))
            .execute(conn)
            .await?;
    let podcast_channels_deleted =
        diesel::delete(podcast_channels::table.filter(podcast_channels::server_id.eq(server_id)))
            .execute(conn)
            .await?;
    let radio_deleted = diesel::delete(
        internet_radio_stations::table.filter(internet_radio_stations::server_id.eq(server_id)),
    )
    .execute(conn)
    .await?;
    tracing::debug!(
        server_id,
        songs = songs_deleted,
        albums = albums_deleted,
        artists = artists_deleted,
        podcast_episodes = podcast_episodes_deleted,
        podcast_channels = podcast_channels_deleted,
        radio_stations = radio_deleted,
        "delete_server_data: completed"
    );
    Ok(())
}

// ── Scan ──────────────────────────────────────────────────────────────────────

/// Scan the upstream server described by `cfg` and upsert its library into the
/// local database.  Caller should have already called `ensure_server_row`.
#[tracing::instrument(skip(conn, cfg), fields(server = %cfg.name, server_id))]
pub async fn scan_server(
    conn: &mut AsyncSqliteConnection,
    cfg: &ServerConfig,
    server_id: i32,
) -> Result<ScanStats, ScanError> {
    let client = Client::new(&cfg.url, Auth::token(&cfg.username, &cfg.password))
        .map_err(|e| ScanError::Client(e.to_string()))?;

    let artist_template = Template::parse(&cfg.matching.artist);
    let album_template = Template::parse(&cfg.matching.album);

    let mut stats = ScanStats::default();

    // ── Artists & Albums & Songs ──────────────────────────────────────────────

    let artists_id3 = client
        .get_artists(None)
        .await
        .map_err(|e| ScanError::Upstream(e.to_string()))?;

    for index in &artists_id3.index {
        for artist_id3 in &index.artist {
            let agg_key = eval_artist(&artist_template, artist_id3);
            let metadata = serde_json::to_string(artist_id3).unwrap_or_default();
            tracing::debug!(artist_id = %artist_id3.id, cover_art = ?artist_id3.cover_art, "storing artist metadata");

            let local_artist_id = upsert_artist(
                conn,
                server_id,
                &artist_id3.id,
                &agg_key,
                &artist_id3.name,
                &metadata,
            )
            .await?;

            stats.artists += 1;

            // Fetch full artist to get album list.
            let artist_with_albums = match client.get_artist(&artist_id3.id).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::warn!(artist_id = %artist_id3.id, error = %e, "getArtist failed, skipping");
                    continue;
                }
            };

            for album_id3 in &artist_with_albums.album {
                let album_agg_key = eval_album(&album_template, album_id3, &agg_key);
                let album_meta = serde_json::to_string(album_id3).unwrap_or_default();
                tracing::debug!(album_id = %album_id3.id, cover_art = ?album_id3.cover_art, "storing album metadata");

                let local_album_id = upsert_album(
                    conn,
                    server_id,
                    &album_id3.id,
                    &album_agg_key,
                    Some(local_artist_id),
                    &album_id3.name,
                    album_id3.year.map(|y| y as i32),
                    album_id3.genre.as_deref(),
                    album_id3.created.as_deref(),
                    album_id3.play_count.map(|p| p as i32),
                    album_id3.played.as_deref(),
                    album_id3.user_rating.map(|r| r as i32),
                    &album_meta,
                )
                .await?;

                stats.albums += 1;

                // Fetch full album to get song list.
                let album_with_songs = match client.get_album(&album_id3.id).await {
                    Ok(a) => a,
                    Err(e) => {
                        tracing::warn!(album_id = %album_id3.id, error = %e, "getAlbum failed, skipping");
                        continue;
                    }
                };

                for song in &album_with_songs.song {
                    let song_meta = serde_json::to_string(song).unwrap_or_default();
                    tracing::debug!(song_id = %song.id, cover_art = ?song.cover_art, "storing song metadata");
                    // Resolve artist FK: prefer the song's own artist_id if present.
                    let song_artist_id = match song.artist_id.as_deref() {
                        Some(uid) => resolve_artist_local_id(conn, server_id, uid)
                            .await
                            .ok()
                            .flatten()
                            .or(Some(local_artist_id)),
                        None => Some(local_artist_id),
                    };

                    upsert_song(
                        conn,
                        server_id,
                        &song.id,
                        Some(local_album_id),
                        song_artist_id,
                        &song.title,
                        song.year.map(|y| y as i32),
                        song.genre.as_deref(),
                        &song_meta,
                    )
                    .await?;

                    stats.songs += 1;
                }
            }
        }
    }

    // ── Podcasts ──────────────────────────────────────────────────────────────

    let podcast_result = client.get_podcasts(Some(true), None).await;
    match podcast_result {
        Ok(channels) => {
            for channel in &channels {
                let ch_meta = serde_json::to_string(channel).unwrap_or_default();
                let local_ch_id = upsert_podcast_channel(
                    conn,
                    server_id,
                    &channel.id,
                    channel.title.as_deref(),
                    &ch_meta,
                )
                .await?;
                stats.podcast_channels += 1;

                for episode in &channel.episode {
                    let ep_meta = serde_json::to_string(episode).unwrap_or_default();
                    upsert_podcast_episode(
                        conn,
                        server_id,
                        &episode.child.id,
                        Some(local_ch_id),
                        Some(episode.child.title.as_str()),
                        episode.publish_date.as_deref(),
                        &ep_meta,
                    )
                    .await?;
                    stats.podcast_episodes += 1;
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "getPodcasts failed, skipping podcasts");
        }
    }

    // ── Internet Radio ────────────────────────────────────────────────────────

    let radio_result = client.get_internet_radio_stations().await;
    match radio_result {
        Ok(stations) => {
            for station in &stations {
                let meta = serde_json::to_string(station).unwrap_or_default();
                upsert_radio_station(
                    conn,
                    server_id,
                    &station.id,
                    &station.name,
                    &station.stream_url,
                    &meta,
                )
                .await?;
                stats.radio_stations += 1;
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "getInternetRadioStations failed, skipping radio");
        }
    }

    // ── Mark scan complete ────────────────────────────────────────────────────

    let now = chrono_now();
    diesel::update(upstream_servers::table.find(server_id))
        .set(upstream_servers::last_scanned_at.eq(&now))
        .execute(conn)
        .await
        .ok();

    tracing::info!(
        server_id,
        artists = stats.artists,
        albums = stats.albums,
        songs = stats.songs,
        podcast_channels = stats.podcast_channels,
        podcast_episodes = stats.podcast_episodes,
        radio_stations = stats.radio_stations,
        "scan_server: completed"
    );
    Ok(stats)
}

// ── Upsert helpers ────────────────────────────────────────────────────────────

async fn upsert_artist(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
    aggregation_key: &str,
    name: &str,
    metadata_json: &str,
) -> QueryResult<i32> {
    tracing::debug!(server_id, upstream_id, aggregation_key, "upsert_artist");
    diesel::insert_into(artists::table)
        .values(NewArtist {
            server_id,
            upstream_id,
            aggregation_key,
            name,
            metadata_json,
        })
        .on_conflict((artists::server_id, artists::upstream_id))
        .do_update()
        .set((
            artists::aggregation_key.eq(aggregation_key),
            artists::name.eq(name),
            artists::metadata_json.eq(metadata_json),
        ))
        .execute(conn)
        .await?;

    artists::table
        .filter(
            artists::server_id
                .eq(server_id)
                .and(artists::upstream_id.eq(upstream_id)),
        )
        .select(artists::id)
        .first(conn)
        .await
}

#[allow(clippy::too_many_arguments)]
async fn upsert_album(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
    aggregation_key: &str,
    artist_id: Option<i32>,
    name: &str,
    year: Option<i32>,
    genre: Option<&str>,
    created_at: Option<&str>,
    play_count: Option<i32>,
    played_at: Option<&str>,
    user_rating: Option<i32>,
    metadata_json: &str,
) -> QueryResult<i32> {
    tracing::debug!(server_id, upstream_id, aggregation_key, "upsert_album");
    diesel::insert_into(albums::table)
        .values(NewAlbum {
            server_id,
            upstream_id,
            aggregation_key,
            artist_id,
            name,
            year,
            genre,
            created_at,
            play_count,
            played_at,
            user_rating,
            metadata_json,
        })
        .on_conflict((albums::server_id, albums::upstream_id))
        .do_update()
        .set((
            albums::aggregation_key.eq(aggregation_key),
            albums::artist_id.eq(artist_id),
            albums::name.eq(name),
            albums::year.eq(year),
            albums::genre.eq(genre),
            albums::created_at.eq(created_at),
            albums::play_count.eq(play_count),
            albums::played_at.eq(played_at),
            albums::user_rating.eq(user_rating),
            albums::metadata_json.eq(metadata_json),
        ))
        .execute(conn)
        .await?;

    albums::table
        .filter(
            albums::server_id
                .eq(server_id)
                .and(albums::upstream_id.eq(upstream_id)),
        )
        .select(albums::id)
        .first(conn)
        .await
}

#[allow(clippy::too_many_arguments)]
async fn upsert_song(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
    album_id: Option<i32>,
    artist_id: Option<i32>,
    title: &str,
    year: Option<i32>,
    genre: Option<&str>,
    metadata_json: &str,
) -> QueryResult<()> {
    tracing::debug!(server_id, upstream_id, "upsert_song");
    diesel::insert_into(songs::table)
        .values(NewSong {
            server_id,
            upstream_id,
            album_id,
            artist_id,
            title,
            year,
            genre,
            metadata_json,
        })
        .on_conflict((songs::server_id, songs::upstream_id))
        .do_update()
        .set((
            songs::album_id.eq(album_id),
            songs::artist_id.eq(artist_id),
            songs::title.eq(title),
            songs::year.eq(year),
            songs::genre.eq(genre),
            songs::metadata_json.eq(metadata_json),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

async fn upsert_podcast_channel(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
    title: Option<&str>,
    metadata_json: &str,
) -> QueryResult<i32> {
    diesel::insert_into(podcast_channels::table)
        .values(NewPodcastChannel {
            server_id,
            upstream_id,
            title,
            metadata_json,
        })
        .on_conflict((podcast_channels::server_id, podcast_channels::upstream_id))
        .do_update()
        .set((
            podcast_channels::title.eq(title),
            podcast_channels::metadata_json.eq(metadata_json),
        ))
        .execute(conn)
        .await?;

    podcast_channels::table
        .filter(
            podcast_channels::server_id
                .eq(server_id)
                .and(podcast_channels::upstream_id.eq(upstream_id)),
        )
        .select(podcast_channels::id)
        .first(conn)
        .await
}

async fn upsert_podcast_episode(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
    channel_id: Option<i32>,
    title: Option<&str>,
    publish_date: Option<&str>,
    metadata_json: &str,
) -> QueryResult<()> {
    diesel::insert_into(podcast_episodes::table)
        .values(NewPodcastEpisode {
            server_id,
            upstream_id,
            channel_id,
            title,
            publish_date,
            metadata_json,
        })
        .on_conflict((podcast_episodes::server_id, podcast_episodes::upstream_id))
        .do_update()
        .set((
            podcast_episodes::channel_id.eq(channel_id),
            podcast_episodes::title.eq(title),
            podcast_episodes::publish_date.eq(publish_date),
            podcast_episodes::metadata_json.eq(metadata_json),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

async fn upsert_radio_station(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
    name: &str,
    stream_url: &str,
    metadata_json: &str,
) -> QueryResult<()> {
    diesel::insert_into(internet_radio_stations::table)
        .values(NewInternetRadioStation {
            server_id,
            upstream_id,
            name,
            stream_url,
            metadata_json,
        })
        .on_conflict((
            internet_radio_stations::server_id,
            internet_radio_stations::upstream_id,
        ))
        .do_update()
        .set((
            internet_radio_stations::name.eq(name),
            internet_radio_stations::stream_url.eq(stream_url),
            internet_radio_stations::metadata_json.eq(metadata_json),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

/// Look up the local `artists.id` for a given `(server_id, upstream_id)` pair.
async fn resolve_artist_local_id(
    conn: &mut AsyncSqliteConnection,
    server_id: i32,
    upstream_id: &str,
) -> QueryResult<Option<i32>> {
    artists::table
        .filter(
            artists::server_id
                .eq(server_id)
                .and(artists::upstream_id.eq(upstream_id)),
        )
        .select(artists::id)
        .first(conn)
        .await
        .optional()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs.to_string()
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Statistics collected during a scan.
#[derive(Debug, Default)]
pub struct ScanStats {
    pub artists: usize,
    pub albums: usize,
    pub songs: usize,
    pub podcast_channels: usize,
    pub podcast_episodes: usize,
    pub radio_stations: usize,
}

/// Errors that can occur during scanning.
#[derive(Debug)]
pub enum ScanError {
    /// Failed to construct the upstream client (e.g. bad URL).
    Client(String),
    /// The upstream server returned an error.
    Upstream(String),
    /// A database operation failed.
    Db(diesel::result::Error),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::Client(s) => write!(f, "client error: {s}"),
            ScanError::Upstream(s) => write!(f, "upstream error: {s}"),
            ScanError::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl From<diesel::result::Error> for ScanError {
    fn from(e: diesel::result::Error) -> Self {
        ScanError::Db(e)
    }
}

// ── Convenience: scan all configured servers ──────────────────────────────────

/// Scan every server in `servers` sequentially, acquiring a fresh DB connection
/// per server.  Errors are logged but do not abort subsequent servers.
pub async fn run_full_scan(pool: &DbPool, servers: &[ServerConfig]) {
    for cfg in servers {
        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(server = %cfg.name, error = %e, "scan: failed to get DB connection");
                continue;
            }
        };

        let server_id = match ensure_server_row(&mut conn, cfg).await {
            Ok((id, _)) => id,
            Err(e) => {
                tracing::error!(server = %cfg.name, error = %e, "scan: failed to ensure server row");
                continue;
            }
        };

        tracing::info!(server = %cfg.name, "scan: starting");
        match scan_server(&mut conn, cfg, server_id).await {
            Ok(stats) => tracing::info!(
                server = %cfg.name,
                artists = stats.artists,
                albums = stats.albums,
                songs = stats.songs,
                podcast_channels = stats.podcast_channels,
                podcast_episodes = stats.podcast_episodes,
                radio_stations = stats.radio_stations,
                "scan: complete",
            ),
            Err(e) => tracing::warn!(server = %cfg.name, error = %e, "scan: failed"),
        }
    }
}
