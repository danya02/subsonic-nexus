use opensubsonic::data::{
    AlbumInfo, AlbumWithSongsId3, ArtistInfo, ArtistInfo2, ArtistWithAlbumsId3, ArtistsId3,
    Child, Directory, Genre, Indexes, MusicFolder,
};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::SubsonicResponse;

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
    pub artists: ArtistsId3,
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

/// The `opensubsonic` crate does not expose a `VideoInfo` type; raw JSON
/// until an appropriate type is available.
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

/// Used by both getAlbumInfo and getAlbumInfo2 (same JSON key).
#[derive(Serialize)]
pub struct AlbumInfoResponse {
    #[serde(rename = "albumInfo")]
    pub album_info: AlbumInfo,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET/POST /rest/getMusicFolders — no extra parameters
pub async fn get_music_folders(_auth: SubsonicAuth) -> SubsonicResponse<MusicFoldersResponse> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<GetIndexesParams>,
) -> SubsonicResponse<IndexesResponse> {
    todo!()
}

// --- getMusicDirectory ---

#[derive(Deserialize)]
pub struct GetMusicDirectoryParams {
    pub id: String,
}

/// GET/POST /rest/getMusicDirectory
pub async fn get_music_directory(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetMusicDirectoryParams>,
) -> SubsonicResponse<DirectoryResponse> {
    todo!()
}

/// GET/POST /rest/getGenres — no extra parameters
pub async fn get_genres(_auth: SubsonicAuth) -> SubsonicResponse<GenresResponse> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<GetArtistsParams>,
) -> SubsonicResponse<ArtistsResponse> {
    todo!()
}

// --- getArtist ---

#[derive(Deserialize)]
pub struct GetArtistParams {
    pub id: String,
}

/// GET/POST /rest/getArtist
pub async fn get_artist(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetArtistParams>,
) -> SubsonicResponse<ArtistResponse> {
    todo!()
}

// --- getAlbum ---

#[derive(Deserialize)]
pub struct GetAlbumParams {
    pub id: String,
}

/// GET/POST /rest/getAlbum
pub async fn get_album(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetAlbumParams>,
) -> SubsonicResponse<AlbumResponse> {
    todo!()
}

// --- getSong ---

#[derive(Deserialize)]
pub struct GetSongParams {
    pub id: String,
}

/// GET/POST /rest/getSong
pub async fn get_song(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetSongParams>,
) -> SubsonicResponse<SongResponse> {
    todo!()
}

/// GET/POST /rest/getVideos — no extra parameters
pub async fn get_videos(_auth: SubsonicAuth) -> SubsonicResponse<VideosResponse> {
    todo!()
}

// --- getVideoInfo ---

#[derive(Deserialize)]
pub struct GetVideoInfoParams {
    pub id: String,
}

/// GET/POST /rest/getVideoInfo
pub async fn get_video_info(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetVideoInfoParams>,
) -> SubsonicResponse<VideoInfoResponse> {
    todo!()
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
    QueryOrForm(_params): QueryOrForm<GetArtistInfoParams>,
) -> SubsonicResponse<ArtistInfoResponse> {
    todo!()
}

/// GET/POST /rest/getArtistInfo2
pub async fn get_artist_info2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetArtistInfoParams>,
) -> SubsonicResponse<ArtistInfo2Response> {
    todo!()
}

// --- getAlbumInfo / getAlbumInfo2 ---

#[derive(Deserialize)]
pub struct GetAlbumInfoParams {
    pub id: String,
}

/// GET/POST /rest/getAlbumInfo
pub async fn get_album_info(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetAlbumInfoParams>,
) -> SubsonicResponse<AlbumInfoResponse> {
    todo!()
}

/// GET/POST /rest/getAlbumInfo2
pub async fn get_album_info2(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetAlbumInfoParams>,
) -> SubsonicResponse<AlbumInfoResponse> {
    todo!()
}
