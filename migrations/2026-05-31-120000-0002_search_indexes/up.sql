-- Speed up search3/search2 ORDER BY + LIKE pagination on large libraries.
-- Without these indexes SQLite does a full table scan + filesort for every page.
CREATE INDEX idx_songs_title  ON songs(title  COLLATE NOCASE);
CREATE INDEX idx_albums_name  ON albums(name   COLLATE NOCASE);
-- artists already has idx_artists_name ON artists(name COLLATE NOCASE)
