CREATE TABLE cover_art_sources (
    upstream_cover_art_id TEXT    NOT NULL,
    server_id             INTEGER NOT NULL REFERENCES upstream_servers(id) ON DELETE CASCADE,
    PRIMARY KEY (upstream_cover_art_id, server_id)
);
