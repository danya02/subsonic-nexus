use axum::extract::State;
use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use opensubsonic::data::{
    AlbumInfo, AlbumWithSongsId3, Artist, ArtistInfo, ArtistInfo2, ArtistWithAlbumsId3, Child,
    Directory, Genre, Index, Indexes, MusicFolder,
};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::db::schema::upstream_servers;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::{
    IdTemplate, album_id3_from_canonical, artist_id3_from_canonical, artists_to_id3_response,
    build_entity_id, child_from_song_row, get_conn, query_album_by_nexus_id,
    query_albums_for_artist, query_artist_by_nexus_id, query_canonical_artists,
    query_song_by_nexus_id, query_songs_for_album,
};
use crate::response::SubsonicResponse;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct MusicFoldersBody {
    #[serde(rename = "musicFolder")]
    pub music_folder: Vec<MusicFolder>,
}

#[derive(Serialize)]
pub struct MusicFoldersResponse {
    #[serde(rename = "musicFolders")]
    pub music_folders: MusicFoldersBody,
}

#[derive(Serialize)]
pub struct IndexesResponse {
    pub indexes: Indexes,
}

#[derive(Serialize)]
pub struct DirectoryResponse {
    pub directory: Directory,
}

#[derive(Serialize)]
pub struct GenresBody {
    pub genre: Vec<Genre>,
}

#[derive(Serialize)]
pub struct GenresResponse {
    pub genres: GenresBody,
}

#[derive(Serialize)]
pub struct ArtistsResponse {
    pub artists: opensubsonic::data::ArtistsId3,
}

#[derive(Serialize)]
pub struct ArtistResponse {
    pub artist: ArtistWithAlbumsId3,
}

#[derive(Serialize)]
pub struct AlbumResponse {
    pub album: AlbumWithSongsId3,
}

#[derive(Serialize)]
pub struct SongResponse {
    pub song: Child,
}

#[derive(Serialize)]
pub struct VideosBody {
    pub video: Vec<Child>,
}

#[derive(Serialize)]
pub struct VideosResponse {
    pub videos: VideosBody,
}

#[derive(Serialize)]
pub struct VideoInfoResponse {
    #[serde(rename = "videoInfo")]
    pub video_info: serde_json::Value,
}

#[derive(Serialize)]
pub struct ArtistInfoResponse {
    #[serde(rename = "artistInfo")]
    pub artist_info: ArtistInfo,
}

#[derive(Serialize)]
pub struct ArtistInfo2Response {
    #[serde(rename = "artistInfo2")]
    pub artist_info2: ArtistInfo2,
}

#[derive(Serialize)]
pub struct AlbumInfoResponse {
    #[serde(rename = "albumInfo")]
    pub album_info: AlbumInfo,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET/POST /rest/getMusicFolders — no extra parameters
pub async fn get_music_folders(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
) -> Result<SubsonicResponse<MusicFoldersResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;

    // `upstream_servers.name` is stored as non-null TEXT in the DB schema.
    let rows: Vec<(i32, String)> = upstream_servers::table
        .select((upstream_servers::id, upstream_servers::name))
        .order(upstream_servers::priority.asc())
        .load(&mut conn)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?;

    tracing::debug!(folder_count = rows.len(), "getMusicFolders");
    let folders = rows
        .into_iter()
        .map(|(id, name)| MusicFolder {
            id: id as i64,
            name: Some(name),
        })
        .collect();

    Ok(MusicFoldersResponse {
        music_folders: MusicFoldersBody {
            music_folder: folders,
        },
    }
    .into())
}

// --- getIndexes ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetIndexesParams {
    pub music_folder_id: Option<String>,
    pub if_modified_since: Option<i64>,
}

/// GET/POST /rest/getIndexes
pub async fn get_indexes(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetIndexesParams>,
) -> Result<SubsonicResponse<IndexesResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let server_filter = params.music_folder_id.and_then(|s| s.parse::<i32>().ok());
    let artists = query_canonical_artists(&mut conn, server_filter)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?;
    tracing::debug!(server_filter = ?server_filter, artist_count = artists.len(), "getIndexes");

    let cfg = &state.config.nexus;
    let template = IdTemplate::from_config(&cfg.entity_id_template);

    let mut index_map: Vec<(String, Vec<Artist>)> = Vec::new();
    for row in &artists {
        let nexus_id = build_entity_id(template, &row.server_name, row.server_id, &row.upstream_id);
        let letter = index_letter(&row.name);
        let legacy_artist = Artist {
            id: nexus_id,
            name: row.name.clone(),
            artist_image_url: None,
            starred: None,
            user_rating: None,
            average_rating: None,
        };
        if let Some(entry) = index_map.iter_mut().find(|(k, _)| k == &letter) {
            entry.1.push(legacy_artist);
        } else {
            index_map.push((letter, vec![legacy_artist]));
        }
    }

    let index = index_map
        .into_iter()
        .map(|(name, artist)| Index { name, artist })
        .collect();

    Ok(IndexesResponse {
        indexes: Indexes {
            ignored_articles: None,
            last_modified: None,
            shortcut: vec![],
            child: vec![],
            index,
        },
    }
    .into())
}

// --- getMusicDirectory ---

#[derive(Deserialize)]
pub struct GetMusicDirectoryParams {
    pub id: String,
}

/// GET/POST /rest/getMusicDirectory
///
/// id is one of:
///   - a server DB id (music folder) → list artists as dirs
///   - a nexus artist id → list albums as dirs
///   - a nexus album id → list songs as children
pub async fn get_music_directory(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetMusicDirectoryParams>,
) -> Result<SubsonicResponse<DirectoryResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let template = IdTemplate::from_config(&cfg.entity_id_template);
    let id = &params.id;
    tracing::info!(id = %id, "getMusicDirectory");

    // Try to interpret as a server (music folder) id first.
    if let Ok(server_id) = id.parse::<i32>() {
        // Check if this is a valid server id.
        let server_exists: bool = upstream_servers::table
            .filter(upstream_servers::id.eq(server_id))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| SubsonicError::generic(e.to_string()))?
            > 0;

        if server_exists {
            let artists = query_canonical_artists(&mut conn, Some(server_id))
                .await
                .map_err(|e| SubsonicError::generic(e.to_string()))?;
            tracing::debug!(
                server_id,
                artist_count = artists.len(),
                "getMusicDirectory: listing artists for server"
            );
            let children: Vec<Child> = artists
                .iter()
                .map(|row| {
                    let nid = build_entity_id(
                        template,
                        &row.server_name,
                        row.server_id,
                        &row.upstream_id,
                    );
                    Child {
                        id: nid,
                        parent: Some(id.clone()),
                        is_dir: true,
                        title: row.name.clone(),
                        ..Child::default_song()
                    }
                })
                .collect();
            return Ok(DirectoryResponse {
                directory: Directory {
                    id: id.clone(),
                    parent: None,
                    name: format!("Server {server_id}"),
                    starred: None,
                    user_rating: None,
                    average_rating: None,
                    play_count: None,
                    child: children,
                },
            }
            .into());
        }
    }

    // Try as artist ID → list albums.
    if let Some(artist) = query_artist_by_nexus_id(&mut conn, template, id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?
    {
        let albums = query_albums_for_artist(&mut conn, &artist.aggregation_key)
            .await
            .map_err(|e| SubsonicError::generic(e.to_string()))?;
        tracing::debug!(artist_name = %artist.name, album_count = albums.len(), "getMusicDirectory: listing albums for artist");
        let children: Vec<Child> = albums
            .iter()
            .map(|row| {
                let nid =
                    build_entity_id(template, &row.server_name, row.server_id, &row.upstream_id);
                let mut al = album_id3_from_canonical(row, cfg);
                Child {
                    id: nid,
                    parent: Some(id.clone()),
                    is_dir: true,
                    title: al.name.clone(),
                    artist: al.artist.clone(),
                    year: al.year,
                    genre: al.genre.clone(),
                    cover_art: al.cover_art.take(),
                    ..Child::default_song()
                }
            })
            .collect();
        return Ok(DirectoryResponse {
            directory: Directory {
                id: id.clone(),
                parent: None,
                name: artist.name.clone(),
                starred: None,
                user_rating: None,
                average_rating: None,
                play_count: None,
                child: children,
            },
        }
        .into());
    }

    // Try as album ID → list songs.
    if let Some(album) = query_album_by_nexus_id(&mut conn, template, id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?
    {
        let songs = query_songs_for_album(&mut conn, album.db_id)
            .await
            .map_err(|e| SubsonicError::generic(e.to_string()))?;
        tracing::debug!(album_name = %album.name, song_count = songs.len(), "getMusicDirectory: listing songs for album");
        let children: Vec<Child> = songs
            .iter()
            .map(|row| {
                let mut c = child_from_song_row(row, cfg);
                c.parent = Some(id.clone());
                c
            })
            .collect();
        return Ok(DirectoryResponse {
            directory: Directory {
                id: id.clone(),
                parent: None,
                name: album.name.clone(),
                starred: None,
                user_rating: None,
                average_rating: None,
                play_count: None,
                child: children,
            },
        }
        .into());
    }

    Err(SubsonicError::not_found(format!(
        "No directory found for id: {id}"
    )))
}

/// GET/POST /rest/getGenres — no extra parameters
pub async fn get_genres(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
) -> Result<SubsonicResponse<GenresResponse>, SubsonicError> {
    use diesel::sql_types::*;
    #[derive(QueryableByName)]
    struct GenreRow {
        #[diesel(sql_type = Text)]
        genre: String,
        #[diesel(sql_type = BigInt)]
        song_count: i64,
    }
    #[derive(QueryableByName)]
    struct AlbumGenreRow {
        #[diesel(sql_type = Text)]
        genre: String,
        #[diesel(sql_type = BigInt)]
        album_count: i64,
    }

    let mut conn = get_conn(&state.pool).await?;

    let song_counts: Vec<GenreRow> = sql_query(
        "SELECT genre, COUNT(*) AS song_count FROM songs
         WHERE genre IS NOT NULL AND genre != ''
         GROUP BY genre ORDER BY genre COLLATE NOCASE",
    )
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let album_counts: Vec<AlbumGenreRow> = sql_query(
        "SELECT genre, COUNT(*) AS album_count FROM albums
         WHERE genre IS NOT NULL AND genre != ''
         GROUP BY genre",
    )
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let genres: Vec<Genre> = song_counts
        .into_iter()
        .map(|row| {
            let album_count = album_counts
                .iter()
                .find(|a| a.genre == row.genre)
                .map_or(0, |a| a.album_count);
            Genre {
                name: row.genre,
                song_count: row.song_count,
                album_count,
            }
        })
        .collect();

    tracing::debug!(genre_count = genres.len(), "getGenres");
    Ok(GenresResponse {
        genres: GenresBody { genre: genres },
    }
    .into())
}

// --- getArtists ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetArtistsParams {
    pub music_folder_id: Option<String>,
}

/// GET/POST /rest/getArtists
pub async fn get_artists(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetArtistsParams>,
) -> Result<SubsonicResponse<ArtistsResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let server_filter = params.music_folder_id.and_then(|s| s.parse::<i32>().ok());
    let rows = query_canonical_artists(&mut conn, server_filter)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let cfg = &state.config.nexus;
    tracing::debug!(server_filter = ?server_filter, artist_count = rows.len(), "getArtists");
    let artists: Vec<_> = rows
        .iter()
        .map(|r| artist_id3_from_canonical(r, cfg))
        .collect();
    let artists_id3 = artists_to_id3_response(artists);

    Ok(ArtistsResponse {
        artists: artists_id3,
    }
    .into())
}

// --- getArtist ---

#[derive(Deserialize)]
pub struct GetArtistParams {
    pub id: String,
}

/// GET/POST /rest/getArtist
pub async fn get_artist(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetArtistParams>,
) -> Result<SubsonicResponse<ArtistResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let template = IdTemplate::from_config(&cfg.entity_id_template);

    tracing::debug!(artist_id = %params.id, "getArtist");
    let row = query_artist_by_nexus_id(&mut conn, template, &params.id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?
        .ok_or_else(|| SubsonicError::not_found(format!("Artist not found: {}", params.id)))?;

    let artist_id3 = artist_id3_from_canonical(&row, cfg);

    let albums = query_albums_for_artist(&mut conn, &row.aggregation_key)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?;
    let album_count = albums.len() as i64;
    tracing::debug!(artist_name = %row.name, album_count, "getArtist: found");
    let album_list: Vec<_> = albums
        .iter()
        .map(|a| album_id3_from_canonical(a, cfg))
        .collect();

    Ok(ArtistResponse {
        artist: ArtistWithAlbumsId3 {
            id: artist_id3.id,
            name: artist_id3.name,
            cover_art: artist_id3.cover_art,
            artist_image_url: artist_id3.artist_image_url,
            album_count: Some(album_count),
            starred: artist_id3.starred,
            music_brainz_id: artist_id3.music_brainz_id,
            sort_name: artist_id3.sort_name,
            roles: artist_id3.roles,
            album: album_list,
        },
    }
    .into())
}

// --- getAlbum ---

#[derive(Deserialize)]
pub struct GetAlbumParams {
    pub id: String,
}

/// GET/POST /rest/getAlbum
pub async fn get_album(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetAlbumParams>,
) -> Result<SubsonicResponse<AlbumResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let template = IdTemplate::from_config(&cfg.entity_id_template);

    tracing::debug!(album_id = %params.id, "getAlbum");
    let row = query_album_by_nexus_id(&mut conn, template, &params.id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?
        .ok_or_else(|| SubsonicError::not_found(format!("Album not found: {}", params.id)))?;

    let al = album_id3_from_canonical(&row, cfg);

    let songs = query_songs_for_album(&mut conn, row.db_id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?;
    let song_list: Vec<Child> = songs.iter().map(|s| child_from_song_row(s, cfg)).collect();
    tracing::debug!(album_name = %row.name, song_count = song_list.len(), "getAlbum: found");

    Ok(AlbumResponse {
        album: AlbumWithSongsId3 {
            id: al.id,
            name: al.name,
            version: al.version,
            artist: al.artist,
            artist_id: al.artist_id,
            cover_art: al.cover_art,
            song_count: Some(song_list.len() as i64),
            duration: al.duration,
            play_count: al.play_count,
            created: al.created,
            starred: al.starred,
            year: al.year,
            genre: al.genre,
            played: al.played,
            user_rating: al.user_rating,
            record_labels: al.record_labels,
            music_brainz_id: al.music_brainz_id,
            genres: al.genres,
            artists: al.artists,
            display_artist: al.display_artist,
            original_release_date: al.original_release_date,
            release_date: al.release_date,
            is_compilation: al.is_compilation,
            sort_name: al.sort_name,
            disc_titles: al.disc_titles,
            explicit_status: al.explicit_status,
            moods: al.moods,
            song: song_list,
        },
    }
    .into())
}

// --- getSong ---

#[derive(Deserialize)]
pub struct GetSongParams {
    pub id: String,
}

/// GET/POST /rest/getSong
pub async fn get_song(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetSongParams>,
) -> Result<SubsonicResponse<SongResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let cfg = &state.config.nexus;
    let template = IdTemplate::from_config(&cfg.entity_id_template);

    tracing::debug!(song_id = %params.id, "getSong");
    let row = query_song_by_nexus_id(&mut conn, template, &params.id)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?
        .ok_or_else(|| SubsonicError::not_found(format!("Song not found: {}", params.id)))?;

    tracing::debug!(song_title = %row.title, "getSong: found");
    Ok(SongResponse {
        song: child_from_song_row(&row, cfg),
    }
    .into())
}

/// GET/POST /rest/getVideos — no extra parameters
pub async fn get_videos(
    _auth: SubsonicAuth,
    _state: State<AppState>,
) -> SubsonicResponse<VideosResponse> {
    VideosResponse {
        videos: VideosBody { video: vec![] },
    }
    .into()
}

// --- getVideoInfo ---

#[derive(Deserialize)]
pub struct GetVideoInfoParams {
    pub id: String,
}

/// GET/POST /rest/getVideoInfo
pub async fn get_video_info(
    _auth: SubsonicAuth,
    _state: State<AppState>,
    QueryOrForm(_params): QueryOrForm<GetVideoInfoParams>,
) -> Result<SubsonicResponse<VideoInfoResponse>, SubsonicError> {
    Err(SubsonicError::not_found("Video not supported"))
}

// --- getArtistInfo / getArtistInfo2 ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetArtistInfoParams {
    pub id: String,
    pub count: Option<i32>,
    pub include_not_present: Option<bool>,
}

/// GET/POST /rest/getArtistInfo
pub async fn get_artist_info(
    _auth: SubsonicAuth,
    _state: State<AppState>,
    QueryOrForm(_params): QueryOrForm<GetArtistInfoParams>,
) -> SubsonicResponse<ArtistInfoResponse> {
    ArtistInfoResponse {
        artist_info: ArtistInfo {
            biography: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
            similar_artist: vec![],
        },
    }
    .into()
}

/// GET/POST /rest/getArtistInfo2
pub async fn get_artist_info2(
    _auth: SubsonicAuth,
    _state: State<AppState>,
    QueryOrForm(_params): QueryOrForm<GetArtistInfoParams>,
) -> SubsonicResponse<ArtistInfo2Response> {
    ArtistInfo2Response {
        artist_info2: ArtistInfo2 {
            biography: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
            similar_artist: vec![],
        },
    }
    .into()
}

// --- getAlbumInfo / getAlbumInfo2 ---

#[derive(Deserialize)]
pub struct GetAlbumInfoParams {
    pub id: String,
}

/// GET/POST /rest/getAlbumInfo
pub async fn get_album_info(
    _auth: SubsonicAuth,
    _state: State<AppState>,
    QueryOrForm(_params): QueryOrForm<GetAlbumInfoParams>,
) -> SubsonicResponse<AlbumInfoResponse> {
    AlbumInfoResponse {
        album_info: AlbumInfo {
            notes: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
        },
    }
    .into()
}

/// GET/POST /rest/getAlbumInfo2
pub async fn get_album_info2(
    _auth: SubsonicAuth,
    _state: State<AppState>,
    QueryOrForm(_params): QueryOrForm<GetAlbumInfoParams>,
) -> SubsonicResponse<AlbumInfoResponse> {
    AlbumInfoResponse {
        album_info: AlbumInfo {
            notes: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
        },
    }
    .into()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

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

fn strip_articles(name: &str) -> &str {
    for prefix in &["the ", "a ", "an "] {
        // Use .get() instead of [] to avoid panicking on multi-byte characters
        // (e.g. an artist name starting with a CJK character whose UTF-8
        // encoding happens to be the same length as a prefix).
        if name
            .get(..prefix.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(prefix))
        {
            return &name[prefix.len()..];
        }
    }
    name
}

// Default-like construction helper for Child (it doesn't derive Default)
trait ChildDefault {
    fn default_song() -> Child;
}
impl ChildDefault for Child {
    fn default_song() -> Child {
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
}
