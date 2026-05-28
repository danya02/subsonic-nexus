use axum::extract::State;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use opensubsonic::data::{SearchResult, SearchResult2, SearchResult3};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::{
    CanonicalAlbum, CanonicalArtist, IdTemplate, SongRow, album_id3_from_canonical,
    artist_id3_from_canonical, child_from_song_row, get_conn,
};
use crate::response::SubsonicResponse;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SearchResultResponse {
    #[serde(rename = "searchResult")]
    pub search_result: SearchResult,
}

#[derive(Serialize)]
pub struct SearchResult2Response {
    #[serde(rename = "searchResult2")]
    pub search_result2: SearchResult2,
}

#[derive(Serialize)]
pub struct SearchResult3Response {
    #[serde(rename = "searchResult3")]
    pub search_result3: SearchResult3,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- search (legacy) ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub any: Option<bool>,
    pub count: Option<i32>,
    pub offset: Option<i32>,
    pub newer_than: Option<i64>,
}

/// GET/POST /rest/search
pub async fn search(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<SearchParams>,
) -> Result<SubsonicResponse<SearchResultResponse>, SubsonicError> {
    // The legacy `search` endpoint is essentially a stripped-down search2.
    // Build a query string from whichever field is set.
    let query = params
        .title
        .or(params.artist)
        .or(params.album)
        .unwrap_or_default();

    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let count = params.count.unwrap_or(20) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    let songs = search_songs(&mut conn, &query, count, offset).await?;
    tracing::debug!(query = %query, song_count = songs.len(), "search: results");
    let song_list: Vec<_> = songs.iter().map(|s| child_from_song_row(s, cfg)).collect();

    Ok(SearchResultResponse {
        search_result: SearchResult {
            offset: Some(offset),
            total_hits: Some(song_list.len() as i64),
            matches: song_list,
        },
    }
    .into())
}

// --- search2 / search3 ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Search2Params {
    pub query: String,
    pub artist_count: Option<i32>,
    pub artist_offset: Option<i32>,
    pub album_count: Option<i32>,
    pub album_offset: Option<i32>,
    pub song_count: Option<i32>,
    pub song_offset: Option<i32>,
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/search2
pub async fn search2(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<Search2Params>,
) -> Result<SubsonicResponse<SearchResult2Response>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;

    let artist_count = params.artist_count.unwrap_or(20) as i64;
    let artist_offset = params.artist_offset.unwrap_or(0) as i64;
    let album_count = params.album_count.unwrap_or(20) as i64;
    let album_offset = params.album_offset.unwrap_or(0) as i64;
    let song_count = params.song_count.unwrap_or(20) as i64;
    let song_offset = params.song_offset.unwrap_or(0) as i64;

    let artists =
        search_canonical_artists(&mut conn, &params.query, artist_count, artist_offset).await?;
    let albums =
        search_canonical_albums(&mut conn, &params.query, album_count, album_offset).await?;
    let songs = search_songs(&mut conn, &params.query, song_count, song_offset).await?;
    tracing::debug!(query = %params.query, artist_count = artists.len(), album_count = albums.len(), song_count = songs.len(), "search2: results");

    let _artist_list: Vec<_> = artists
        .iter()
        .map(|a| artist_id3_from_canonical(a, cfg))
        .collect();
    // search2 returns albums as Child-like (legacy), but the opensubsonic crate uses ArtistId3
    // for artists. For search2 the `artist` field is Vec<Artist> (legacy type).
    // Map ArtistId3 → legacy Artist.
    let legacy_artists: Vec<opensubsonic::data::Artist> = artists
        .iter()
        .map(|r| {
            let nid = crate::nexus::build_entity_id(
                IdTemplate::from_config(&cfg.entity_id_template),
                &r.server_name,
                r.server_id,
                &r.upstream_id,
            );
            opensubsonic::data::Artist {
                id: nid,
                name: r.name.clone(),
                artist_image_url: None,
                starred: None,
                user_rating: None,
                average_rating: None,
            }
        })
        .collect();

    let album_children: Vec<opensubsonic::data::Child> = albums
        .iter()
        .map(|r| {
            let al = album_id3_from_canonical(r, cfg);
            opensubsonic::data::Child {
                id: al.id.clone(),
                parent: None,
                is_dir: true,
                title: al.name.clone(),
                artist: al.artist.clone(),
                year: al.year,
                genre: al.genre.clone(),
                cover_art: al.cover_art.clone(),
                ..child_default()
            }
        })
        .collect();

    let song_list = songs.iter().map(|s| child_from_song_row(s, cfg)).collect();

    Ok(SearchResult2Response {
        search_result2: SearchResult2 {
            artist: legacy_artists,
            album: album_children,
            song: song_list,
        },
    }
    .into())
}

/// GET/POST /rest/search3
pub async fn search3(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<Search2Params>,
) -> Result<SubsonicResponse<SearchResult3Response>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;

    let artist_count = params.artist_count.unwrap_or(20) as i64;
    let artist_offset = params.artist_offset.unwrap_or(0) as i64;
    let album_count = params.album_count.unwrap_or(20) as i64;
    let album_offset = params.album_offset.unwrap_or(0) as i64;
    let song_count = params.song_count.unwrap_or(20) as i64;
    let song_offset = params.song_offset.unwrap_or(0) as i64;

    let artists =
        search_canonical_artists(&mut conn, &params.query, artist_count, artist_offset).await?;
    let albums =
        search_canonical_albums(&mut conn, &params.query, album_count, album_offset).await?;
    let songs = search_songs(&mut conn, &params.query, song_count, song_offset).await?;
    tracing::debug!(query = %params.query, artist_count = artists.len(), album_count = albums.len(), song_count = songs.len(), "search3: results");

    let artist_list: Vec<_> = artists
        .iter()
        .map(|a| artist_id3_from_canonical(a, cfg))
        .collect();
    let album_list: Vec<_> = albums
        .iter()
        .map(|a| album_id3_from_canonical(a, cfg))
        .collect();
    let song_list: Vec<_> = songs.iter().map(|s| child_from_song_row(s, cfg)).collect();

    Ok(SearchResult3Response {
        search_result3: SearchResult3 {
            artist: artist_list,
            album: album_list,
            song: song_list,
        },
    }
    .into())
}

// ---------------------------------------------------------------------------
// Internal search helpers
// ---------------------------------------------------------------------------

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

async fn search_canonical_artists(
    conn: &mut crate::db::AsyncSqliteConnection,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<CanonicalArtist>, SubsonicError> {
    tracing::debug!(query, limit, offset, "search_canonical_artists");
    let q = query
        .replace('\'', "''")
        .replace('%', "\\%")
        .replace('_', "\\_");
    sql_query(format!(
        "{CANONICAL_ARTIST_SQL} AND ar.name LIKE '%{q}%' ESCAPE '\\' COLLATE NOCASE \
         ORDER BY ar.name COLLATE NOCASE LIMIT {limit} OFFSET {offset}"
    ))
    .load(conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))
}

async fn search_canonical_albums(
    conn: &mut crate::db::AsyncSqliteConnection,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<CanonicalAlbum>, SubsonicError> {
    tracing::debug!(query, limit, offset, "search_canonical_albums");
    let q = query
        .replace('\'', "''")
        .replace('%', "\\%")
        .replace('_', "\\_");
    sql_query(format!(
        "{CANONICAL_ALBUM_SQL} AND al.name LIKE '%{q}%' ESCAPE '\\' COLLATE NOCASE \
         ORDER BY al.name COLLATE NOCASE LIMIT {limit} OFFSET {offset}"
    ))
    .load(conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))
}

async fn search_songs(
    conn: &mut crate::db::AsyncSqliteConnection,
    query: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<SongRow>, SubsonicError> {
    tracing::debug!(query, limit, offset, "search_songs");
    let q = query
        .replace('\'', "''")
        .replace('%', "\\%")
        .replace('_', "\\_");
    sql_query(format!(
        r#"SELECT
            so.id       AS db_id,
            so.server_id,
            s.name      AS server_name,
            so.upstream_id,
            so.title,
            so.metadata_json
        FROM songs so
        JOIN upstream_servers s ON so.server_id = s.id
        WHERE so.title LIKE '%{q}%' ESCAPE '\' COLLATE NOCASE
        ORDER BY so.title COLLATE NOCASE
        LIMIT {limit} OFFSET {offset}"#
    ))
    .load(conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))
}

fn child_default() -> opensubsonic::data::Child {
    opensubsonic::data::Child {
        id: String::new(),
        parent: None,
        is_dir: false,
        title: String::new(),
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
    }
}
