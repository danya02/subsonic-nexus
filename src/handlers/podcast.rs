use opensubsonic::data::{PodcastChannel, PodcastEpisode};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct PodcastsBody {
    pub channel: Vec<PodcastChannel>,
}

#[derive(Serialize)]
pub struct PodcastsResponse {
    pub podcasts: PodcastsBody,
}

#[derive(Serialize)]
pub struct NewestPodcastsBody {
    pub episode: Vec<PodcastEpisode>,
}

#[derive(Serialize)]
pub struct NewestPodcastsResponse {
    #[serde(rename = "newestPodcasts")]
    pub newest_podcasts: NewestPodcastsBody,
}

#[derive(Serialize)]
pub struct PodcastEpisodeResponse {
    pub episode: PodcastEpisode,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- getPodcasts ---

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPodcastsParams {
    pub include_episodes: Option<bool>,
    pub id: Option<String>,
}

/// GET/POST /rest/getPodcasts
pub async fn get_podcasts(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetPodcastsParams>,
) -> SubsonicResponse<PodcastsResponse> {
    todo!()
}

// --- getNewestPodcasts ---

#[derive(Deserialize)]
pub struct GetNewestPodcastsParams {
    pub count: Option<i32>,
}

/// GET/POST /rest/getNewestPodcasts
pub async fn get_newest_podcasts(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetNewestPodcastsParams>,
) -> SubsonicResponse<NewestPodcastsResponse> {
    todo!()
}

// --- getPodcastEpisode ---

#[derive(Deserialize)]
pub struct GetPodcastEpisodeParams {
    pub id: String,
}

/// GET/POST /rest/getPodcastEpisode
pub async fn get_podcast_episode(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetPodcastEpisodeParams>,
) -> SubsonicResponse<PodcastEpisodeResponse> {
    todo!()
}

// --- createPodcastChannel ---

#[derive(Deserialize)]
pub struct CreatePodcastChannelParams {
    pub url: String,
}

/// GET/POST /rest/createPodcastChannel
pub async fn create_podcast_channel(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<CreatePodcastChannelParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- deletePodcastChannel ---

#[derive(Deserialize)]
pub struct DeletePodcastChannelParams {
    pub id: String,
}

/// GET/POST /rest/deletePodcastChannel
pub async fn delete_podcast_channel(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeletePodcastChannelParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- deletePodcastEpisode ---

#[derive(Deserialize)]
pub struct DeletePodcastEpisodeParams {
    pub id: String,
}

/// GET/POST /rest/deletePodcastEpisode
pub async fn delete_podcast_episode(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeletePodcastEpisodeParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

// --- downloadPodcastEpisode ---

#[derive(Deserialize)]
pub struct DownloadPodcastEpisodeParams {
    pub id: String,
}

/// GET/POST /rest/downloadPodcastEpisode
pub async fn download_podcast_episode(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DownloadPodcastEpisodeParams>,
) -> SubsonicResponse<Empty> {
    todo!()
}

/// GET/POST /rest/refreshPodcasts — no extra parameters
pub async fn refresh_podcasts(_auth: SubsonicAuth) -> SubsonicResponse<Empty> {
    todo!()
}
