//! Nexus core utilities: entity ID engine, canonical DB queries, and type
//! converters from DB rows → opensubsonic response types.
//!
//! This module is the central hub for aggregation logic.  All handlers
//! should use the helpers here rather than writing their own SQL.

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel_async::RunQueryDsl;
use opensubsonic::data::{AlbumId3, ArtistId3, ArtistsId3, Child, IndexId3, MusicFolder};

use crate::config::{NexusConfig, ServerConfig};
use crate::db::AsyncSqliteConnection;
use crate::error::SubsonicError;

// ── Entity ID engine ──────────────────────────────────────────────────────────

/// The three supported entity-ID template forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdTemplate {
    /// `{upstream_id}` — use the upstream's own ID directly.
    UpstreamOnly,
    /// `{server_name}:{upstream_id}` — prefix with the server name.
    ServerNamePrefix,
    /// `{server_id}:{upstream_id}` — prefix with the server's DB row id.
    ServerIdPrefix,
}

impl IdTemplate {
    /// Detect the template variant from the config string.
    pub fn from_config(s: &str) -> Self {
        if s.contains("{server_name}") {
            Self::ServerNamePrefix
        } else if s.contains("{server_id}") {
            Self::ServerIdPrefix
        } else {
            Self::UpstreamOnly
        }
    }
}

/// Build a nexus entity ID from the canonical server row.
pub fn build_entity_id(
    template: IdTemplate,
    server_name: &str,
    server_db_id: i32,
    upstream_id: &str,
) -> String {
    match template {
        IdTemplate::UpstreamOnly => upstream_id.to_owned(),
        IdTemplate::ServerNamePrefix => format!("{server_name}:{upstream_id}"),
        IdTemplate::ServerIdPrefix => format!("{server_db_id}:{upstream_id}"),
    }
}

/// A parsed nexus entity ID.
pub struct ParsedId {
    /// Optional hint about which server this ID belongs to.
    pub server_hint: Option<ServerHint>,
    /// The upstream entity ID on the canonical server.
    pub upstream_id: String,
}

/// How a server was identified in a prefixed nexus ID.
pub enum ServerHint {
    Name(String),
    DbId(i32),
}

/// Parse a nexus entity ID back into its components.
pub fn parse_entity_id(template: IdTemplate, nexus_id: &str) -> ParsedId {
    match template {
        IdTemplate::UpstreamOnly => ParsedId {
            server_hint: None,
            upstream_id: nexus_id.to_owned(),
        },
        IdTemplate::ServerNamePrefix | IdTemplate::ServerIdPrefix => {
            if let Some(colon) = nexus_id.find(':') {
                let prefix = &nexus_id[..colon];
                let upstream = &nexus_id[colon + 1..];
                let hint = match template {
                    IdTemplate::ServerNamePrefix => Some(ServerHint::Name(prefix.to_owned())),
                    IdTemplate::ServerIdPrefix => {
                        prefix.parse::<i32>().ok().map(ServerHint::DbId)
                    }
                    _ => unreachable!(),
                };
                ParsedId { server_hint: hint, upstream_id: upstream.to_owned() }
            } else {
                // Malformed — fall back to treating the whole string as upstream ID.
                ParsedId { server_hint: None, upstream_id: nexus_id.to_owned() }
            }
        }
    }
}

/// Build a cover-art ID.  Always uses `{server_db_id}:{upstream_cover_art}` format
/// regardless of the entity-ID template, because cover art needs the server
/// to be unambiguous for proxying.
pub fn build_cover_art_id(server_db_id: i32, upstream_cover_art: &str) -> String {
    format!("{server_db_id}:{upstream_cover_art}")
}

/// Parse a nexus cover-art ID back to `(server_db_id, upstream_cover_art_id)`.
pub fn parse_cover_art_id(nexus_id: &str) -> Option<(i32, &str)> {
    let colon = nexus_id.find(':')?;
    let server_id = nexus_id[..colon].parse::<i32>().ok()?;
    let cover_art = &nexus_id[colon + 1..];
    Some((server_id, cover_art))
}

// ── Upstream URL builder ──────────────────────────────────────────────────────

/// Build an authenticated upstream REST URL.
///
/// Uses token+salt auth (Subsonic spec §6.2):
/// `t = md5(password + salt)`, `s = random hex salt`
pub fn build_upstream_url(
    cfg: &ServerConfig,
    endpoint: &str,
    extra_params: &[(&str, &str)],
) -> String {
    use std::fmt::Write;
    let salt = random_hex_salt();
    let token = format!("{:x}", md5::compute(format!("{}{}", cfg.password, salt)));

    let mut url = format!(
        "{}/rest/{}?u={}&t={}&s={}&v=1.16.1&c=subsonic-nexus&f=json",
        cfg.url.trim_end_matches('/'),
        endpoint,
        urlencoded(&cfg.username),
        urlencoded(&token),
        urlencoded(&salt),
    );
    for (k, v) in extra_params {
        let _ = write!(url, "&{}={}", k, urlencoded(v));
    }
    url
}

fn random_hex_salt() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Simple pseudo-random salt from time + process id.
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let p = std::process::id();
    format!("{:08x}{:08x}", t, p)
}

fn urlencoded(s: &str) -> String {
    // Percent-encode everything that's not unreserved.
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || "-._~".contains(c) {
                vec![c]
            } else {
                let bytes = c.to_string().into_bytes();
                bytes
                    .into_iter()
                    .flat_map(|b| format!("%{b:02X}").chars().collect::<Vec<_>>())
                    .collect()
            }
        })
        .collect()
}

// ── Canonical DB row types ────────────────────────────────────────────────────

/// A canonical (deduplicated) artist row, joined with its server name.
#[derive(QueryableByName, Debug, Clone)]
pub struct CanonicalArtist {
    #[diesel(sql_type = Integer)]
    pub db_id: i32,
    #[diesel(sql_type = Integer)]
    pub server_id: i32,
    #[diesel(sql_type = Text)]
    pub server_name: String,
    #[diesel(sql_type = Text)]
    pub upstream_id: String,
    #[diesel(sql_type = Text)]
    pub aggregation_key: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub metadata_json: String,
}

/// A canonical (deduplicated) album row, joined with its server name.
#[derive(QueryableByName, Debug, Clone)]
pub struct CanonicalAlbum {
    #[diesel(sql_type = Integer)]
    pub db_id: i32,
    #[diesel(sql_type = Integer)]
    pub server_id: i32,
    #[diesel(sql_type = Text)]
    pub server_name: String,
    #[diesel(sql_type = Text)]
    pub upstream_id: String,
    #[diesel(sql_type = Text)]
    pub aggregation_key: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub artist_upstream_id_opt: Option<i32>, // not used directly; metadata_json has it
    #[diesel(sql_type = Text)]
    pub metadata_json: String,
}

/// A song row (songs are not deduplicated), joined with server name.
#[derive(QueryableByName, Debug, Clone)]
pub struct SongRow {
    #[diesel(sql_type = Integer)]
    pub db_id: i32,
    #[diesel(sql_type = Integer)]
    pub server_id: i32,
    #[diesel(sql_type = Text)]
    pub server_name: String,
    #[diesel(sql_type = Text)]
    pub upstream_id: String,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = Text)]
    pub metadata_json: String,
}

// ── Canonical DB queries ──────────────────────────────────────────────────────

const CANONICAL_ARTIST_SQL: &str = r#"
    SELECT
        ar.id       AS db_id,
        ar.server_id,
        s.name      AS server_name,
        ar.upstream_id,
        ar.aggregation_key,
        ar.name,
        ar.metadata_json
    FROM artists ar
    JOIN upstream_servers s ON ar.server_id = s.id
    WHERE s.priority = (
        SELECT MIN(s2.priority)
        FROM artists ar2
        JOIN upstream_servers s2 ON ar2.server_id = s2.id
        WHERE ar2.aggregation_key = ar.aggregation_key
    )
"#;

/// Fetch all canonical artists, ordered by name.
/// If `server_db_id` is given, restrict to that server only (music-folder filter).
pub async fn query_canonical_artists(
    conn: &mut AsyncSqliteConnection,
    server_db_id: Option<i32>,
) -> Result<Vec<CanonicalArtist>, diesel::result::Error> {
    if let Some(sid) = server_db_id {
        sql_query(format!(
            "{CANONICAL_ARTIST_SQL} AND ar.server_id = {sid} ORDER BY ar.name COLLATE NOCASE"
        ))
        .load(conn)
        .await
    } else {
        sql_query(format!("{CANONICAL_ARTIST_SQL} ORDER BY ar.name COLLATE NOCASE"))
            .load(conn)
            .await
    }
}

/// Fetch the canonical artist matching a nexus entity ID.
pub async fn query_artist_by_nexus_id(
    conn: &mut AsyncSqliteConnection,
    template: IdTemplate,
    nexus_id: &str,
) -> Result<Option<CanonicalArtist>, diesel::result::Error> {
    let parsed = parse_entity_id(template, nexus_id);
    let upstream_id = &parsed.upstream_id;

    // Build WHERE clause depending on whether we have a server hint.
    let where_extra = match &parsed.server_hint {
        None => String::new(),
        Some(ServerHint::Name(name)) => {
            format!(" AND s.name = '{}'", name.replace('\'', "''"))
        }
        Some(ServerHint::DbId(id)) => format!(" AND ar.server_id = {id}"),
    };

    let uid = upstream_id.replace('\'', "''");
    let rows: Vec<CanonicalArtist> = sql_query(format!(
        "{CANONICAL_ARTIST_SQL} AND ar.upstream_id = '{uid}'{where_extra} LIMIT 1"
    ))
    .load(conn)
    .await?;
    Ok(rows.into_iter().next())
}

const CANONICAL_ALBUM_SQL: &str = r#"
    SELECT
        al.id       AS db_id,
        al.server_id,
        s.name      AS server_name,
        al.upstream_id,
        al.aggregation_key,
        al.name,
        NULL        AS artist_upstream_id_opt,
        al.metadata_json
    FROM albums al
    JOIN upstream_servers s ON al.server_id = s.id
    WHERE s.priority = (
        SELECT MIN(s2.priority)
        FROM albums al2
        JOIN upstream_servers s2 ON al2.server_id = s2.id
        WHERE al2.aggregation_key = al.aggregation_key
    )
"#;

/// Fetch canonical albums for a given artist `aggregation_key`.
pub async fn query_albums_for_artist(
    conn: &mut AsyncSqliteConnection,
    artist_agg_key: &str,
) -> Result<Vec<CanonicalAlbum>, diesel::result::Error> {
    let key = artist_agg_key.replace('\'', "''");
    sql_query(format!(
        r#"{CANONICAL_ALBUM_SQL}
        AND al.id IN (
            SELECT al3.id FROM albums al3
            JOIN artists ar3 ON al3.artist_id = ar3.id
            WHERE ar3.aggregation_key = '{key}'
        )
        ORDER BY al.year NULLS LAST, al.name COLLATE NOCASE"#
    ))
    .load(conn)
    .await
}

/// Fetch a canonical album by nexus entity ID.
pub async fn query_album_by_nexus_id(
    conn: &mut AsyncSqliteConnection,
    template: IdTemplate,
    nexus_id: &str,
) -> Result<Option<CanonicalAlbum>, diesel::result::Error> {
    let parsed = parse_entity_id(template, nexus_id);
    let uid = parsed.upstream_id.replace('\'', "''");

    let where_extra = match &parsed.server_hint {
        None => String::new(),
        Some(ServerHint::Name(name)) => {
            format!(" AND s.name = '{}'", name.replace('\'', "''"))
        }
        Some(ServerHint::DbId(id)) => format!(" AND al.server_id = {id}"),
    };

    let rows: Vec<CanonicalAlbum> = sql_query(format!(
        "{CANONICAL_ALBUM_SQL} AND al.upstream_id = '{uid}'{where_extra} LIMIT 1"
    ))
    .load(conn)
    .await?;
    Ok(rows.into_iter().next())
}

/// Fetch songs for a given canonical album (by its DB id).
pub async fn query_songs_for_album(
    conn: &mut AsyncSqliteConnection,
    album_db_id: i32,
) -> Result<Vec<SongRow>, diesel::result::Error> {
    sql_query(format!(
        r#"
        SELECT
            so.id       AS db_id,
            so.server_id,
            s.name      AS server_name,
            so.upstream_id,
            so.title,
            so.metadata_json
        FROM songs so
        JOIN upstream_servers s ON so.server_id = s.id
        WHERE so.album_id = {album_db_id}
        ORDER BY
            CAST(json_extract(so.metadata_json, '$.discNumber') AS INTEGER) NULLS LAST,
            CAST(json_extract(so.metadata_json, '$.track') AS INTEGER) NULLS LAST,
            so.title COLLATE NOCASE
        "#
    ))
    .load(conn)
    .await
}

/// Fetch a song by nexus entity ID.
pub async fn query_song_by_nexus_id(
    conn: &mut AsyncSqliteConnection,
    template: IdTemplate,
    nexus_id: &str,
) -> Result<Option<SongRow>, diesel::result::Error> {
    let parsed = parse_entity_id(template, nexus_id);
    let uid = parsed.upstream_id.replace('\'', "''");

    let where_server = match &parsed.server_hint {
        None => String::new(),
        Some(ServerHint::Name(n)) => format!(" AND s.name = '{}'", n.replace('\'', "''")),
        Some(ServerHint::DbId(id)) => format!(" AND so.server_id = {id}"),
    };

    let rows: Vec<SongRow> = sql_query(format!(
        r#"
        SELECT
            so.id       AS db_id,
            so.server_id,
            s.name      AS server_name,
            so.upstream_id,
            so.title,
            so.metadata_json
        FROM songs so
        JOIN upstream_servers s ON so.server_id = s.id
        WHERE so.upstream_id = '{uid}'{where_server}
        LIMIT 1
        "#
    ))
    .load(conn)
    .await?;
    Ok(rows.into_iter().next())
}

// ── Type converters ───────────────────────────────────────────────────────────

/// Convert a canonical artist DB row to an `ArtistId3`, rewriting the entity
/// ID and cover-art ID to use nexus IDs.
pub fn artist_id3_from_canonical(row: &CanonicalArtist, cfg: &NexusConfig) -> ArtistId3 {
    let template = IdTemplate::from_config(&cfg.entity_id_template);
    let nexus_id =
        build_entity_id(template, &row.server_name, row.server_id, &row.upstream_id);

    let mut a: ArtistId3 =
        serde_json::from_str(&row.metadata_json).unwrap_or_else(|_| ArtistId3 {
            id: nexus_id.clone(),
            name: row.name.clone(),
            cover_art: None,
            artist_image_url: None,
            album_count: None,
            starred: None,
            music_brainz_id: None,
            sort_name: None,
            roles: None,
        });

    a.id = nexus_id;
    if let Some(ref ca) = a.cover_art.clone() {
        a.cover_art = Some(build_cover_art_id(row.server_id, ca));
    }
    a
}

/// Convert a canonical album DB row to an `AlbumId3`, rewriting IDs.
pub fn album_id3_from_canonical(row: &CanonicalAlbum, cfg: &NexusConfig) -> AlbumId3 {
    let template = IdTemplate::from_config(&cfg.entity_id_template);
    let nexus_id =
        build_entity_id(template, &row.server_name, row.server_id, &row.upstream_id);

    let mut al: AlbumId3 =
        serde_json::from_str(&row.metadata_json).unwrap_or_else(|_| AlbumId3 {
            id: nexus_id.clone(),
            name: row.name.clone(),
            version: None,
            artist: None,
            artist_id: None,
            cover_art: None,
            song_count: None,
            duration: None,
            play_count: None,
            created: None,
            starred: None,
            year: None,
            genre: None,
            played: None,
            user_rating: None,
            record_labels: None,
            music_brainz_id: None,
            genres: None,
            artists: None,
            display_artist: None,
            release_types: None,
            original_release_date: None,
            release_date: None,
            is_compilation: None,
            sort_name: None,
            disc_titles: None,
            explicit_status: None,
            moods: None,
        });

    al.id = nexus_id;
    if let Some(ref ca) = al.cover_art.clone() {
        al.cover_art = Some(build_cover_art_id(row.server_id, ca));
    }
    // Rewrite artist_id to nexus ID if it's an upstream ID.
    // With UpstreamOnly template this is already correct.
    if let Some(ref aid) = al.artist_id.clone() {
        if template != IdTemplate::UpstreamOnly {
            al.artist_id = Some(build_entity_id(
                template,
                &row.server_name,
                row.server_id,
                aid,
            ));
        }
    }
    al
}

/// Convert a song DB row to a `Child`, rewriting IDs.
pub fn child_from_song_row(row: &SongRow, cfg: &NexusConfig) -> Child {
    let template = IdTemplate::from_config(&cfg.entity_id_template);
    let nexus_id =
        build_entity_id(template, &row.server_name, row.server_id, &row.upstream_id);

    let mut child: Child =
        serde_json::from_str(&row.metadata_json).unwrap_or_else(|_| Child {
            id: nexus_id.clone(),
            parent: None,
            is_dir: false,
            title: row.title.clone(),
            album: None,
            artist: None,
            track: None,
            year: None,
            genre: None,
            cover_art: None,
            size: None,
            content_type: None,
            suffix: None,
            transcoded_content_type: None,
            transcoded_suffix: None,
            duration: None,
            bit_rate: None,
            bit_depth: None,
            sampling_rate: None,
            channel_count: None,
            path: None,
            is_video: None,
            user_rating: None,
            average_rating: None,
            play_count: None,
            disc_number: None,
            created: None,
            starred: None,
            album_id: None,
            artist_id: None,
            media_type_generic: None,
            media_type: None,
            bookmark_position: None,
            original_width: None,
            original_height: None,
            played: None,
            bpm: None,
            comment: None,
            sort_name: None,
            music_brainz_id: None,
            isrc: None,
            genres: None,
            artists: None,
            display_artist: None,
            album_artists: None,
            display_album_artist: None,
            contributors: None,
            display_composer: None,
            moods: None,
            replay_gain: None,
            explicit_status: None,
            works: None,
            movements: None,
        });

    child.id = nexus_id;
    if let Some(ref ca) = child.cover_art.clone() {
        child.cover_art = Some(build_cover_art_id(row.server_id, ca));
    }
    if template != IdTemplate::UpstreamOnly {
        if let Some(ref aid) = child.artist_id.clone() {
            child.artist_id =
                Some(build_entity_id(template, &row.server_name, row.server_id, aid));
        }
        if let Some(ref alid) = child.album_id.clone() {
            child.album_id =
                Some(build_entity_id(template, &row.server_name, row.server_id, alid));
        }
    }
    child
}

// ── Grouping helpers ──────────────────────────────────────────────────────────

/// Group a list of `ArtistId3` into `IndexId3` entries by first character,
/// returning an `ArtistsId3` response payload.
pub fn artists_to_id3_response(artists: Vec<ArtistId3>) -> ArtistsId3 {
    let mut map: Vec<(String, Vec<ArtistId3>)> = Vec::new();

    for artist in artists {
        let key = index_letter(&artist.name);
        if let Some(entry) = map.iter_mut().find(|(k, _)| k == &key) {
            entry.1.push(artist);
        } else {
            map.push((key, vec![artist]));
        }
    }

    let index = map
        .into_iter()
        .map(|(name, artist)| IndexId3 { name, artist })
        .collect();

    ArtistsId3 { ignored_articles: None, index }
}

/// Derive the index letter (A–Z or "#") for an artist name.
fn index_letter(name: &str) -> String {
    let stripped = strip_articles(name);
    let first = stripped
        .chars()
        .next()
        .or_else(|| name.chars().next())
        .unwrap_or('#');
    if first.is_ascii_alphabetic() {
        first.to_ascii_uppercase().to_string()
    } else {
        "#".to_string()
    }
}

/// Strip common leading articles (The, A, An) for sorting/indexing.
fn strip_articles(name: &str) -> &str {
    for prefix in &["the ", "a ", "an "] {
        let plen = prefix.len();
        if name.len() > plen
            && name.is_char_boundary(plen)
            && name[..plen].eq_ignore_ascii_case(prefix)
        {
            return &name[plen..];
        }
    }
    name
}

// ── Music folder helpers ──────────────────────────────────────────────────────

/// Convert upstream server DB rows to `MusicFolder` list.
pub fn music_folders_from_servers(
    servers: &[(i32, String)], // (db_id, name)
) -> Vec<MusicFolder> {
    servers
        .iter()
        .map(|(id, name)| MusicFolder {
            id: *id as i64,
            name: Some(name.clone()),
        })
        .collect()
}

// ── Error helpers ─────────────────────────────────────────────────────────────

/// Get a connection from the pool, mapping errors to `SubsonicError`.
pub async fn get_conn(
    pool: &crate::db::DbPool,
) -> Result<
    diesel_async::pooled_connection::bb8::PooledConnection<'_, AsyncSqliteConnection>,
    SubsonicError,
> {
    pool.get()
        .await
        .map_err(|e| SubsonicError::generic(format!("DB pool error: {e}")))
}
