use axum::extract::State;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use opensubsonic::data::{AlbumId3, Child, NowPlayingEntry};
use opensubsonic::{Starred2Content, StarredContent};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::{
    CanonicalAlbum, SongRow, album_id3_from_canonical, child_from_song_row, get_conn,
};
use crate::response::SubsonicResponse;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Shared body types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct SongsBody {
    pub song: Vec<Child>,
}

#[derive(Serialize)]
pub struct ChildAlbumsBody {
    pub album: Vec<Child>,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AlbumListResponse {
    #[serde(rename = "albumList")]
    pub album_list: ChildAlbumsBody,
}

#[derive(Serialize)]
pub struct AlbumList2Body {
    pub album: Vec<AlbumId3>,
}

#[derive(Serialize)]
pub struct AlbumList2Response {
    #[serde(rename = "albumList2")]
    pub album_list2: AlbumList2Body,
}

#[derive(Serialize)]
pub struct RandomSongsResponse {
    #[serde(rename = "randomSongs")]
    pub random_songs: SongsBody,
}

#[derive(Serialize)]
pub struct SongsByGenreResponse {
    #[serde(rename = "songsByGenre")]
    pub songs_by_genre: SongsBody,
}

#[derive(Serialize)]
pub struct NowPlayingBody {
    pub entry: Vec<NowPlayingEntry>,
}

#[derive(Serialize)]
pub struct NowPlayingResponse {
    #[serde(rename = "nowPlaying")]
    pub now_playing: NowPlayingBody,
}

#[derive(Serialize)]
pub struct StarredResponse {
    pub starred: StarredContent,
}

#[derive(Serialize)]
pub struct Starred2Response {
    pub starred2: Starred2Content,
}

#[derive(Serialize)]
pub struct SimilarSongsResponse {
    #[serde(rename = "similarSongs")]
    pub similar_songs: SongsBody,
}

#[derive(Serialize)]
pub struct SimilarSongs2Response {
    #[serde(rename = "similarSongs2")]
    pub similar_songs2: SongsBody,
}

#[derive(Serialize)]
pub struct TopSongsResponse {
    #[serde(rename = "topSongs")]
    pub top_songs: SongsBody,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- getAlbumList / getAlbumList2 ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAlbumListParams {
    #[serde(rename = "type")]
    pub list_type: String,
    pub size: Option<i32>,
    pub offset: Option<i32>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub genre: Option<String>,
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/getAlbumList
pub async fn get_album_list(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetAlbumListParams>,
) -> Result<SubsonicResponse<AlbumListResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let albums = fetch_album_list(&mut conn, &params).await?;

    let album_children: Vec<Child> = albums
        .iter()
        .map(|row| {
            let al = album_id3_from_canonical(row, cfg);
            Child {
                id: al.id,
                parent: None,
                is_dir: true,
                title: al.name,
                artist: al.artist,
                year: al.year,
                genre: al.genre,
                cover_art: al.cover_art,
                ..child_default()
            }
        })
        .collect();

    Ok(AlbumListResponse {
        album_list: ChildAlbumsBody { album: album_children },
    }
    .into())
}

/// GET/POST /rest/getAlbumList2
pub async fn get_album_list2(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetAlbumListParams>,
) -> Result<SubsonicResponse<AlbumList2Response>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let albums = fetch_album_list(&mut conn, &params).await?;

    let album_list: Vec<AlbumId3> =
        albums.iter().map(|row| album_id3_from_canonical(row, cfg)).collect();

    Ok(AlbumList2Response { album_list2: AlbumList2Body { album: album_list } }.into())
}

// --- getRandomSongs ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRandomSongsParams {
    pub size: Option<i32>,
    pub from_year: Option<i32>,
    pub to_year: Option<i32>,
    pub genre: Option<String>,
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/getRandomSongs
pub async fn get_random_songs(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetRandomSongsParams>,
) -> Result<SubsonicResponse<RandomSongsResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;

    let size = params.size.unwrap_or(10).clamp(1, 500) as i64;
    let mut where_clauses = Vec::<String>::new();

    if let Some(fy) = params.from_year {
        where_clauses.push(format!("so.year >= {fy}"));
    }
    if let Some(ty) = params.to_year {
        where_clauses.push(format!("so.year <= {ty}"));
    }
    if let Some(ref g) = params.genre {
        let g = g.replace('\'', "''");
        where_clauses.push(format!("so.genre = '{g}'"));
    }
    if let Some(ref fid) = params.music_folder_id {
        if let Ok(sid) = fid.parse::<i32>() {
            where_clauses.push(format!("so.server_id = {sid}"));
        }
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("AND {}", where_clauses.join(" AND "))
    };

    let songs: Vec<SongRow> = sql_query(format!(
        r#"SELECT so.id AS db_id, so.server_id, s.name AS server_name,
               so.upstream_id, so.title, so.metadata_json
           FROM songs so
           JOIN upstream_servers s ON so.server_id = s.id
           WHERE 1=1 {where_sql}
           ORDER BY RANDOM()
           LIMIT {size}"#
    ))
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let song_list: Vec<Child> = songs.iter().map(|s| child_from_song_row(s, cfg)).collect();
    Ok(RandomSongsResponse { random_songs: SongsBody { song: song_list } }.into())
}

// --- getSongsByGenre ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSongsByGenreParams {
    pub genre: String,
    pub count: Option<i32>,
    pub offset: Option<i32>,
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/getSongsByGenre
pub async fn get_songs_by_genre(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetSongsByGenreParams>,
) -> Result<SubsonicResponse<SongsByGenreResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;

    let count = params.count.unwrap_or(10).clamp(1, 500) as i64;
    let offset = params.offset.unwrap_or(0) as i64;
    let genre = params.genre.replace('\'', "''");

    let songs: Vec<SongRow> = sql_query(format!(
        r#"SELECT so.id AS db_id, so.server_id, s.name AS server_name,
               so.upstream_id, so.title, so.metadata_json
           FROM songs so
           JOIN upstream_servers s ON so.server_id = s.id
           WHERE so.genre = '{genre}' COLLATE NOCASE
           ORDER BY so.title COLLATE NOCASE
           LIMIT {count} OFFSET {offset}"#
    ))
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let song_list: Vec<Child> = songs.iter().map(|s| child_from_song_row(s, cfg)).collect();
    Ok(SongsByGenreResponse { songs_by_genre: SongsBody { song: song_list } }.into())
}

/// GET/POST /rest/getNowPlaying — no extra parameters
pub async fn get_now_playing(_auth: SubsonicAuth) -> SubsonicResponse<NowPlayingResponse> {
    NowPlayingResponse { now_playing: NowPlayingBody { entry: vec![] } }.into()
}

// --- getStarred / getStarred2 ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStarredParams {
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/getStarred
pub async fn get_starred(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetStarredParams>,
) -> SubsonicResponse<StarredResponse> {
    StarredResponse {
        starred: StarredContent { artist: vec![], album: vec![], song: vec![] },
    }
    .into()
}

/// GET/POST /rest/getStarred2
pub async fn get_starred2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetStarredParams>,
) -> SubsonicResponse<Starred2Response> {
    Starred2Response {
        starred2: Starred2Content { artist: vec![], album: vec![], song: vec![] },
    }
    .into()
}

// --- getSimilarSongs / getSimilarSongs2 ---

#[derive(Deserialize)]
pub struct GetSimilarSongsParams {
    pub id: String,
    pub count: Option<i32>,
}

/// GET/POST /rest/getSimilarSongs
pub async fn get_similar_songs(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetSimilarSongsParams>,
) -> SubsonicResponse<SimilarSongsResponse> {
    SimilarSongsResponse { similar_songs: SongsBody { song: vec![] } }.into()
}

/// GET/POST /rest/getSimilarSongs2
pub async fn get_similar_songs2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetSimilarSongsParams>,
) -> SubsonicResponse<SimilarSongs2Response> {
    SimilarSongs2Response { similar_songs2: SongsBody { song: vec![] } }.into()
}

// --- getTopSongs ---

#[derive(Deserialize)]
pub struct GetTopSongsParams {
    pub id: String,
    pub count: Option<i32>,
}

/// GET/POST /rest/getTopSongs
pub async fn get_top_songs(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetTopSongsParams>,
) -> SubsonicResponse<TopSongsResponse> {
    TopSongsResponse { top_songs: SongsBody { song: vec![] } }.into()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

// Always LEFT JOIN artists so that ORDER BY ar_name.name works for
// alphabeticalByArtist without needing a post-WHERE JOIN (which is invalid SQL).
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
    LEFT JOIN artists ar_name ON al.artist_id = ar_name.id
    WHERE s.priority = (
        SELECT MIN(s2.priority)
        FROM albums al2
        JOIN upstream_servers s2 ON al2.server_id = s2.id
        WHERE al2.aggregation_key = al.aggregation_key
    )
"#;

async fn fetch_album_list(
    conn: &mut crate::db::AsyncSqliteConnection,
    params: &GetAlbumListParams,
) -> Result<Vec<CanonicalAlbum>, SubsonicError> {
    let size = params.size.unwrap_or(10).clamp(1, 500) as i64;
    let offset = params.offset.unwrap_or(0) as i64;

    let order = match params.list_type.as_str() {
        "newest" => "al.created_at DESC NULLS LAST".to_owned(),
        "alphabeticalByName" => "al.name COLLATE NOCASE ASC".to_owned(),
        "alphabeticalByArtist" => {
            "ar_name.name COLLATE NOCASE ASC, al.name COLLATE NOCASE ASC".to_owned()
        }
        "byYear" => "al.year ASC NULLS LAST, al.name COLLATE NOCASE ASC".to_owned(),
        "byGenre" => "al.genre COLLATE NOCASE ASC, al.name COLLATE NOCASE ASC".to_owned(),
        "frequent" => "al.play_count DESC NULLS LAST".to_owned(),
        "recent" => "al.played_at DESC NULLS LAST".to_owned(),
        "starred" => {
            // No star data — return empty.
            return Ok(vec![]);
        }
        _ /* "random" and unknown */ => "RANDOM()".to_owned(),
    };

    let mut extra_where = Vec::<String>::new();

    if params.list_type == "byYear" {
        if let Some(fy) = params.from_year {
            extra_where.push(format!("al.year >= {fy}"));
        }
        if let Some(ty) = params.to_year {
            extra_where.push(format!("al.year <= {ty}"));
        }
    }
    if params.list_type == "byGenre" {
        if let Some(ref g) = params.genre {
            let g = g.replace('\'', "''");
            extra_where.push(format!("al.genre = '{g}' COLLATE NOCASE"));
        }
    }
    if let Some(ref fid) = params.music_folder_id {
        if let Ok(sid) = fid.parse::<i32>() {
            extra_where.push(format!("al.server_id = {sid}"));
        }
    }

    let where_extra = if extra_where.is_empty() {
        String::new()
    } else {
        format!("AND {}", extra_where.join(" AND "))
    };

    sql_query(format!(
        r#"{CANONICAL_ALBUM_SQL} {where_extra}
           ORDER BY {order}
           LIMIT {size} OFFSET {offset}"#
    ))
    .load(conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))
}

fn child_default() -> Child {
    Child {
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
