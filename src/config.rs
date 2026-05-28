//! Configuration types for `nexus.toml`.
//!
//! Credentials (username, password) live here and are never written to the DB.
//! Templates are written to the DB after each scan so the app can detect
//! changes that require a full rescan.

use serde::Deserialize;

/// The full contents of `nexus.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "server", default)]
    pub servers: Vec<ServerConfig>,
    #[serde(default)]
    pub nexus: NexusConfig,
}

/// Per-upstream-server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
    /// Lower value = higher priority (used when aggregating entities across servers).
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub matching: MatchingConfig,
}

/// Aggregation-key templates for this server.
///
/// Template syntax:
///   `{field}`           — insert the field value (null/empty → "")
///   `{field|transform}` — apply a transform: `lowercase`, `trim`,
///                         `ascii_normalize`, `strip_articles`
///   `{field:-fallback}` — use `fallback` expression if field is null/empty
///
/// Artist fields: `music_brainz_id`, `name`, `sort_name`
/// Album fields:  `music_brainz_id`, `name`, `sort_name`, `year`,
///                `display_artist`, `artist_key` (the artist's computed key)
#[derive(Debug, Clone, Deserialize)]
pub struct MatchingConfig {
    #[serde(default = "default_artist_template")]
    pub artist: String,
    #[serde(default = "default_album_template")]
    pub album: String,
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            artist: default_artist_template(),
            album: default_album_template(),
        }
    }
}

fn default_artist_template() -> String {
    "{music_brainz_id:-{name|lowercase|trim}}".to_owned()
}

fn default_album_template() -> String {
    "{music_brainz_id:-{artist_key}:{name|lowercase|trim}}".to_owned()
}

/// Nexus-wide settings (the `[nexus]` table in `nexus.toml`).
#[derive(Debug, Clone, Deserialize)]
pub struct NexusConfig {
    /// Template used to build the entity ID exposed to Subsonic clients.
    ///
    /// Available fields: `upstream_id`, `server_name`, `server_id`.
    ///
    /// The template must be round-trippable (no transforms) so that the nexus
    /// can parse incoming IDs back to `(server, upstream_id)` pairs.
    ///
    /// Supported forms:
    ///   `{upstream_id}`                  — use the upstream ID directly (default)
    ///   `{server_name}:{upstream_id}`    — prefix with server name
    ///   `{server_id}:{upstream_id}`      — prefix with server DB row id
    #[serde(default = "default_entity_id_template")]
    pub entity_id_template: String,

    /// When `true`, stream/download/getCoverArt byte-proxy responses through
    /// the nexus.  When `false` (default), return HTTP 302 redirects to the
    /// upstream server.
    #[serde(default)]
    pub proxy: bool,

    /// Name of the server to proxy write operations (star, playlists, …) to.
    /// Must match a `[[server]]` `name` field.  If absent, write operations
    /// return `not_authorized`.
    pub write_target: Option<String>,
}

impl Default for NexusConfig {
    fn default() -> Self {
        Self {
            entity_id_template: default_entity_id_template(),
            proxy: false,
            write_target: None,
        }
    }
}

fn default_entity_id_template() -> String {
    "{upstream_id}".to_owned()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: vec![],
            nexus: NexusConfig::default(),
        }
    }
}

impl Config {
    /// Load and parse `nexus.toml` from the given path.
    /// Returns an empty config if the file doesn't exist.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match std::fs::read_to_string(path) {
            Ok(text) => Ok(toml::from_str(&text)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Look up a server config by name.
    pub fn server_by_name(&self, name: &str) -> Option<&ServerConfig> {
        self.servers.iter().find(|s| s.name == name)
    }
}
