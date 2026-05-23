use opensubsonic::data::{AlbumId3, Child, NowPlayingEntry};
use opensubsonic::{Starred2Content, StarredContent};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::SubsonicResponse;

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
    /// Album list type (random, newest, highest, frequent, recent,
    /// alphabeticalByName, alphabeticalByArtist, starred, byYear, byGenre).
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
    QueryOrForm(_params): QueryOrForm<GetAlbumListParams>,
) -> SubsonicResponse<AlbumListResponse> {
    todo!()
}

/// GET/POST /rest/getAlbumList2
pub async fn get_album_list2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetAlbumListParams>,
) -> SubsonicResponse<AlbumList2Response> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<GetRandomSongsParams>,
) -> SubsonicResponse<RandomSongsResponse> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<GetSongsByGenreParams>,
) -> SubsonicResponse<SongsByGenreResponse> {
    todo!()
}

/// GET/POST /rest/getNowPlaying — no extra parameters
pub async fn get_now_playing(_auth: SubsonicAuth) -> SubsonicResponse<NowPlayingResponse> {
    todo!()
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
    todo!()
}

/// GET/POST /rest/getStarred2
pub async fn get_starred2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetStarredParams>,
) -> SubsonicResponse<Starred2Response> {
    todo!()
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
    todo!()
}

/// GET/POST /rest/getSimilarSongs2
pub async fn get_similar_songs2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetSimilarSongsParams>,
) -> SubsonicResponse<SimilarSongs2Response> {
    todo!()
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
    todo!()
}
