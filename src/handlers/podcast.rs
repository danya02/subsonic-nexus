use axum::extract::State;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text};
use diesel_async::RunQueryDsl;
use opensubsonic::data::{PodcastChannel, PodcastEpisode, PodcastStatus};
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::nexus::get_conn;
use crate::response::{Empty, SubsonicResponse};
use crate::state::AppState;

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
// DB rows
// ---------------------------------------------------------------------------

#[derive(diesel::QueryableByName)]
struct ChannelRow {
    #[diesel(sql_type = Integer)]
    pub db_id: i32,
    #[diesel(sql_type = Text)]
    pub upstream_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub title: Option<String>,
    #[diesel(sql_type = Text)]
    pub metadata_json: String,
}

#[derive(diesel::QueryableByName)]
struct EpisodeRow {
    #[diesel(sql_type = Integer)]
    pub db_id: i32,
    #[diesel(sql_type = Text)]
    pub upstream_id: String,
    #[diesel(sql_type = Text)]
    pub channel_upstream_id: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub title: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub publish_date: Option<String>,
    #[diesel(sql_type = Text)]
    pub metadata_json: String,
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
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetPodcastsParams>,
) -> Result<SubsonicResponse<PodcastsResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let include_episodes = params.include_episodes.unwrap_or(true);

    let id_filter = if let Some(ref id) = params.id {
        let id = id.replace('\'', "''");
        format!("AND pc.upstream_id = '{id}'")
    } else {
        String::new()
    };

    let channel_rows: Vec<ChannelRow> = sql_query(format!(
        "SELECT pc.id AS db_id, pc.upstream_id, pc.title, pc.metadata_json \
         FROM podcast_channels pc \
         WHERE 1=1 {id_filter} \
         ORDER BY pc.title COLLATE NOCASE"
    ))
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let mut channels: Vec<PodcastChannel> = channel_rows
        .into_iter()
        .map(|row| {
            serde_json::from_str::<PodcastChannel>(&row.metadata_json).unwrap_or_else(|_| {
                PodcastChannel {
                    id: row.upstream_id,
                    url: String::new(),
                    title: row.title,
                    description: None,
                    cover_art: None,
                    original_image_url: None,
                    status: PodcastStatus::Completed,
                    error_message: None,
                    episode: vec![],
                }
            })
        })
        .collect();

    if include_episodes && !channels.is_empty() {
        // Load all episodes and attach them to their channels
        let episode_rows: Vec<EpisodeRow> = sql_query(
            "SELECT pe.id AS db_id, pe.upstream_id, \
                    pc.upstream_id AS channel_upstream_id, \
                    pe.title, pe.publish_date, pe.metadata_json \
             FROM podcast_episodes pe \
             JOIN podcast_channels pc ON pe.channel_id = pc.id \
             ORDER BY pe.publish_date DESC NULLS LAST",
        )
        .load(&mut conn)
        .await
        .map_err(|e| SubsonicError::generic(e.to_string()))?;

        // Group episodes by channel
        for channel in &mut channels {
            channel.episode = episode_rows
                .iter()
                .filter(|e| e.channel_upstream_id == channel.id)
                .map(episode_from_row)
                .collect();
        }
    }

    Ok(PodcastsResponse {
        podcasts: PodcastsBody { channel: channels },
    }
    .into())
}

// --- getNewestPodcasts ---

#[derive(Deserialize)]
pub struct GetNewestPodcastsParams {
    pub count: Option<i32>,
}

/// GET/POST /rest/getNewestPodcasts
pub async fn get_newest_podcasts(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetNewestPodcastsParams>,
) -> Result<SubsonicResponse<NewestPodcastsResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let count = params.count.unwrap_or(20) as i64;

    let episode_rows: Vec<EpisodeRow> = sql_query(format!(
        "SELECT pe.id AS db_id, pe.upstream_id, \
                pc.upstream_id AS channel_upstream_id, \
                pe.title, pe.publish_date, pe.metadata_json \
         FROM podcast_episodes pe \
         JOIN podcast_channels pc ON pe.channel_id = pc.id \
         ORDER BY pe.publish_date DESC NULLS LAST \
         LIMIT {count}"
    ))
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let episodes: Vec<PodcastEpisode> = episode_rows.iter().map(episode_from_row).collect();
    Ok(NewestPodcastsResponse {
        newest_podcasts: NewestPodcastsBody { episode: episodes },
    }
    .into())
}

// --- getPodcastEpisode ---

#[derive(Deserialize)]
pub struct GetPodcastEpisodeParams {
    pub id: String,
}

/// GET/POST /rest/getPodcastEpisode
pub async fn get_podcast_episode(
    _auth: SubsonicAuth,
    State(state): State<AppState>,
    QueryOrForm(params): QueryOrForm<GetPodcastEpisodeParams>,
) -> Result<SubsonicResponse<PodcastEpisodeResponse>, SubsonicError> {
    let mut conn = get_conn(&state.pool).await?;
    let id = params.id.replace('\'', "''");

    let rows: Vec<EpisodeRow> = sql_query(format!(
        "SELECT pe.id AS db_id, pe.upstream_id, \
                pc.upstream_id AS channel_upstream_id, \
                pe.title, pe.publish_date, pe.metadata_json \
         FROM podcast_episodes pe \
         JOIN podcast_channels pc ON pe.channel_id = pc.id \
         WHERE pe.upstream_id = '{id}'"
    ))
    .load(&mut conn)
    .await
    .map_err(|e| SubsonicError::generic(e.to_string()))?;

    let row = rows.into_iter().next().ok_or_else(|| {
        SubsonicError::not_found(format!("Podcast episode not found: {}", params.id))
    })?;

    Ok(PodcastEpisodeResponse {
        episode: episode_from_row(&row),
    }
    .into())
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized(
        "Podcast management is not supported",
    ))
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized(
        "Podcast management is not supported",
    ))
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized(
        "Podcast management is not supported",
    ))
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
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized(
        "Podcast management is not supported",
    ))
}

/// GET/POST /rest/refreshPodcasts — no extra parameters
pub async fn refresh_podcasts(
    _auth: SubsonicAuth,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized(
        "Podcast management is not supported",
    ))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn episode_from_row(row: &EpisodeRow) -> PodcastEpisode {
    serde_json::from_str::<PodcastEpisode>(&row.metadata_json).unwrap_or_else(|_| {
        use opensubsonic::data::Child;
        PodcastEpisode {
            child: Child {
                id: row.upstream_id.clone(),
                parent: None,
                is_dir: false,
                title: row.title.clone().unwrap_or_default(),
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
            },
            stream_id: None,
            channel_id: row.channel_upstream_id.clone(),
            description: None,
            status: PodcastStatus::Completed,
            publish_date: row.publish_date.clone(),
        }
    })
}
