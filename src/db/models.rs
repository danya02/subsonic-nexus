//! Diesel ORM model types — one `Queryable` + one `Insertable` pair per table.
//!
//! All "extra" upstream metadata (cover art IDs, MusicBrainz IDs, sort names,
//! bitrate, replay gain, …) is stored as a JSON blob in `metadata_json` and
//! deserialized only when building response payloads.

use diesel::prelude::*;
use crate::db::schema::*;

// ── upstream_servers ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = upstream_servers)]
pub struct UpstreamServer {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub priority: i32,
    pub last_scanned_at: Option<String>,
    pub artist_key_template: Option<String>,
    pub album_key_template: Option<String>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = upstream_servers)]
pub struct NewUpstreamServer<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub priority: i32,
    pub last_scanned_at: Option<&'a str>,
    pub artist_key_template: Option<&'a str>,
    pub album_key_template: Option<&'a str>,
}

// ── artists ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = artists)]
#[diesel(belongs_to(UpstreamServer, foreign_key = server_id))]
pub struct Artist {
    pub id: i32,
    pub server_id: i32,
    pub upstream_id: String,
    pub aggregation_key: String,
    pub name: String,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = artists)]
pub struct NewArtist<'a> {
    pub server_id: i32,
    pub upstream_id: &'a str,
    pub aggregation_key: &'a str,
    pub name: &'a str,
    pub metadata_json: &'a str,
}

// ── albums ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = albums)]
#[diesel(belongs_to(UpstreamServer, foreign_key = server_id))]
#[diesel(belongs_to(Artist, foreign_key = artist_id))]
pub struct Album {
    pub id: i32,
    pub server_id: i32,
    pub upstream_id: String,
    pub aggregation_key: String,
    pub artist_id: Option<i32>,
    pub name: String,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub created_at: Option<String>,
    pub play_count: Option<i32>,
    pub played_at: Option<String>,
    pub user_rating: Option<i32>,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = albums)]
pub struct NewAlbum<'a> {
    pub server_id: i32,
    pub upstream_id: &'a str,
    pub aggregation_key: &'a str,
    pub artist_id: Option<i32>,
    pub name: &'a str,
    pub year: Option<i32>,
    pub genre: Option<&'a str>,
    pub created_at: Option<&'a str>,
    pub play_count: Option<i32>,
    pub played_at: Option<&'a str>,
    pub user_rating: Option<i32>,
    pub metadata_json: &'a str,
}

// ── songs ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = songs)]
#[diesel(belongs_to(UpstreamServer, foreign_key = server_id))]
#[diesel(belongs_to(Album, foreign_key = album_id))]
#[diesel(belongs_to(Artist, foreign_key = artist_id))]
pub struct Song {
    pub id: i32,
    pub server_id: i32,
    pub upstream_id: String,
    pub album_id: Option<i32>,
    pub artist_id: Option<i32>,
    pub title: String,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = songs)]
pub struct NewSong<'a> {
    pub server_id: i32,
    pub upstream_id: &'a str,
    pub album_id: Option<i32>,
    pub artist_id: Option<i32>,
    pub title: &'a str,
    pub year: Option<i32>,
    pub genre: Option<&'a str>,
    pub metadata_json: &'a str,
}

// ── podcast_channels ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = podcast_channels)]
#[diesel(belongs_to(UpstreamServer, foreign_key = server_id))]
pub struct PodcastChannel {
    pub id: i32,
    pub server_id: i32,
    pub upstream_id: String,
    pub title: Option<String>,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = podcast_channels)]
pub struct NewPodcastChannel<'a> {
    pub server_id: i32,
    pub upstream_id: &'a str,
    pub title: Option<&'a str>,
    pub metadata_json: &'a str,
}

// ── podcast_episodes ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = podcast_episodes)]
#[diesel(belongs_to(UpstreamServer, foreign_key = server_id))]
#[diesel(belongs_to(PodcastChannel, foreign_key = channel_id))]
pub struct PodcastEpisode {
    pub id: i32,
    pub server_id: i32,
    pub upstream_id: String,
    pub channel_id: Option<i32>,
    pub title: Option<String>,
    pub publish_date: Option<String>,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = podcast_episodes)]
pub struct NewPodcastEpisode<'a> {
    pub server_id: i32,
    pub upstream_id: &'a str,
    pub channel_id: Option<i32>,
    pub title: Option<&'a str>,
    pub publish_date: Option<&'a str>,
    pub metadata_json: &'a str,
}

// ── internet_radio_stations ──────────────────────────────────────────────────

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = internet_radio_stations)]
#[diesel(belongs_to(UpstreamServer, foreign_key = server_id))]
pub struct InternetRadioStation {
    pub id: i32,
    pub server_id: i32,
    pub upstream_id: String,
    pub name: String,
    pub stream_url: String,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = internet_radio_stations)]
pub struct NewInternetRadioStation<'a> {
    pub server_id: i32,
    pub upstream_id: &'a str,
    pub name: &'a str,
    pub stream_url: &'a str,
    pub metadata_json: &'a str,
}
