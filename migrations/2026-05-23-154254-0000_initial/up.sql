-- subsonic-nexus initial schema
--
-- Column philosophy: only fields required for querying, filtering, or sorting
-- get their own columns. Everything else lives in metadata_json and is
-- deserialized only when constructing the response payload.

CREATE TABLE upstream_servers (
    id                    INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name                  TEXT    NOT NULL,
    url                   TEXT    NOT NULL,
    -- Credentials are NOT stored here; they live only in nexus.toml
    priority              INTEGER NOT NULL DEFAULT 0,   -- lower = higher priority
    last_scanned_at       TEXT,
    -- Snapshot of templates used during the last scan.
    -- If these differ from nexus.toml on startup, the server's data is wiped and rescanned.
    artist_key_template   TEXT,
    album_key_template    TEXT
);

-- One row per (server, upstream_artist). aggregation_key is computed at scan time
-- from the server's artist_key_template and stored here for fast grouping.
CREATE TABLE artists (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    upstream_id     TEXT    NOT NULL,
    aggregation_key TEXT    NOT NULL,
    name            TEXT    NOT NULL,   -- for text search and alphabetical listing
    metadata_json   TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(server_id, upstream_id)
);
CREATE INDEX idx_artists_agg_key ON artists(aggregation_key);
CREATE INDEX idx_artists_name    ON artists(name COLLATE NOCASE);

-- One row per (server, upstream_album).
CREATE TABLE albums (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    upstream_id     TEXT    NOT NULL,
    aggregation_key TEXT    NOT NULL,
    artist_id       INTEGER REFERENCES artists(id),  -- FK to primary artist source row
    name            TEXT    NOT NULL,   -- alphabeticalByName, text search
    year            INTEGER,            -- byYear filter
    genre           TEXT,               -- byGenre filter (legacy single genre)
    created_at      TEXT,               -- newest sort
    play_count      INTEGER,            -- frequent sort
    played_at       TEXT,               -- recent sort
    user_rating     INTEGER,            -- highest sort
    metadata_json   TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(server_id, upstream_id)
);
CREATE INDEX idx_albums_agg_key ON albums(aggregation_key);
CREATE INDEX idx_albums_artist  ON albums(artist_id);
CREATE INDEX idx_albums_year    ON albums(year);
CREATE INDEX idx_albums_genre   ON albums(genre);

-- Songs are NOT deduplicated; each row belongs to exactly one album source row.
CREATE TABLE songs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    upstream_id     TEXT    NOT NULL,
    album_id        INTEGER REFERENCES albums(id),
    artist_id       INTEGER REFERENCES artists(id),  -- primary artist FK
    title           TEXT    NOT NULL,   -- text search
    year            INTEGER,            -- getRandomSongs year filter
    genre           TEXT,               -- getSongsByGenre / getRandomSongs
    metadata_json   TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(server_id, upstream_id)
);
CREATE INDEX idx_songs_album  ON songs(album_id);
CREATE INDEX idx_songs_artist ON songs(artist_id);
CREATE INDEX idx_songs_genre  ON songs(genre);

-- Podcasts: one row per (server, upstream_channel); no deduplication.
CREATE TABLE podcast_channels (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    upstream_id     TEXT    NOT NULL,
    title           TEXT,
    metadata_json   TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(server_id, upstream_id)
);

CREATE TABLE podcast_episodes (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    upstream_id     TEXT    NOT NULL,
    channel_id      INTEGER REFERENCES podcast_channels(id) ON DELETE CASCADE,
    title           TEXT,
    publish_date    TEXT,               -- getNewestPodcasts ordering
    metadata_json   TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(server_id, upstream_id)
);

-- Internet radio: one row per (server, upstream_station); no deduplication.
CREATE TABLE internet_radio_stations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id       INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    upstream_id     TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    stream_url      TEXT    NOT NULL,
    metadata_json   TEXT    NOT NULL DEFAULT '{}',
    UNIQUE(server_id, upstream_id)
);
