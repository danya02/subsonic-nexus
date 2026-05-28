# subsonic-nexus — Developer Guide

## What this is

An **OpenSubsonic aggregation proxy**: it pulls the full library from multiple upstream
Navidrome / OpenSubsonic servers, merges them into a single virtual server (with
deduplication for artists and albums), and presents a standard OpenSubsonic API to
clients.

The database (`nexus.db`) is a **cache** of upstream metadata — it can be dropped and
recreated from scratch at any time by re-running a scan. All design decisions account
for this.

---

## Key design decisions

### 1. Entity IDs must survive database recreation

Because `nexus.db` is a disposable cache, the auto-increment integer PKs (`artists.id`,
`albums.id`, `songs.id`) are **not** stable across recreations. Clients that cache IDs
(e.g. Symfonium, DSub) would break if IDs change.

**Solution**: Expose the **upstream server's own ID** as the nexus entity ID (default),
since that ID is stable as long as the upstream server's data doesn't change.

The format is configurable in `nexus.toml` via `[nexus] entity_id_template`:

| Template | Example nexus ID | Notes |
|----------|-----------------|-------|
| `{upstream_id}` (default) | `3u02xYMDYATNDjhzcwITEk` | Works when upstream IDs are unique across all servers (true for Navidrome — IDs are derived from MBIDs or name hashes) |
| `{server_name}:{upstream_id}` | `danya02:3u02xYMDYATNDjhzcwITEk` | Use when servers may produce colliding IDs |
| `{server_id}:{upstream_id}` | `1:3u02xYMDYATNDjhzcwITEk` | Like above but uses DB row id; slightly shorter |

Templates must be **round-trippable** (no transforms). Parsing: if the template contains
`server_name` or `server_id`, split on the first `:`.

Cover art IDs use a separate cover-art template; default: `{server_id}:{upstream_cover_art_id}`.

### 2. Deduplication: canonical row selection

Multiple upstream servers may have the same artist or album. They are linked by a
computed **`aggregation_key`** (stored in the `artists` and `albums` tables).

The **canonical row** for a given `aggregation_key` is the one from the upstream server
with the **lowest `priority` value** (i.e. highest-priority server as configured). This
row provides the metadata shown to clients.

SQL pattern used throughout `src/nexus.rs`:
```sql
SELECT ar.*
FROM artists ar
JOIN upstream_servers s ON ar.server_id = s.id
WHERE s.priority = (
    SELECT MIN(s2.priority)
    FROM artists ar2
    JOIN upstream_servers s2 ON ar2.server_id = s2.id
    WHERE ar2.aggregation_key = ar.aggregation_key
)
```

Songs do **not** have an `aggregation_key` — they are never deduplicated. Songs shown
for an album come from the canonical server's album row.

### 3. `aggregation_key` stability

The `aggregation_key` is computed at scan time from the templates defined in
`nexus.toml` under each `[[server]]` block:
- `matching.artist` (default: `{music_brainz_id:-{name|lowercase|trim}}`)
- `matching.album` (default: `{music_brainz_id:-{artist_key}:{name|lowercase|trim}}`)

If these templates change, all data for that server is wiped and a fresh scan runs.
Store them carefully — changing them invalidates all clients' cached IDs too (because
the canonical row may change, changing the nexus ID).

### 4. Async database access

The project uses **`diesel-async`** with `SyncConnectionWrapper<SqliteConnection>` and
a **`bb8`** connection pool. SQLite itself is synchronous; `SyncConnectionWrapper`
internally spawns blocking threads but presents an async API.

Pool type alias (in `src/db/mod.rs`):
```rust
pub type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncSqliteConnection>>;
pub type AsyncSqliteConnection = SyncConnectionWrapper<diesel::sqlite::SqliteConnection>;
```

Getting a connection in a handler:
```rust
let mut conn = state.pool.get().await.map_err(|e| SubsonicError::generic(e.to_string()))?;
```

### 5. Media delivery (stream / download / getCoverArt)

Configured by `[nexus] proxy = false` (default) or `proxy = true`:

- **`proxy = false`** (default): return HTTP 302 redirect to the upstream URL with
  token+salt auth params. The client connects directly to the upstream server. Simple,
  no extra bandwidth on the nexus.
- **`proxy = true`**: fetch bytes from upstream with `reqwest` and stream them back.
  Use when clients can't reach upstream servers directly.

Upstream auth URL construction (token+salt, spec §6.2):
```
{server_url}/rest/{endpoint}?u={username}&t={token}&s={salt}&v=1.16.1&c=subsonic-nexus&f=json&{extra_params}
```
where `salt` is a random hex string and `token = md5(password + salt)`.

### 6. Mutation / write operations

Write operations are proxied to a configurable `write_target` server (by name):
```toml
[nexus]
write_target = "danya02"
```

- **Scrobble / reportPlayback**: proxy to the server encoded in the song's nexus ID
  (the song's natural canonical server). No `write_target` needed.
- **Star / unstar / setRating**: proxy to `write_target`; translate nexus IDs to that
  server's upstream IDs before forwarding.
- **Playlists, play queue, bookmarks**: proxy all CRUD to `write_target`.
- **User management**: always returns `not_authorized` (not supported).
- If no `write_target` configured, all mutation operations return `not_authorized`.

---

## Repository layout

```
src/
├── main.rs              — server startup, pool init, router
├── router.rs            — all route registrations
├── state.rs             — AppState { pool, config }
├── config.rs            — nexus.toml parsing (Config, ServerConfig, NexusConfig)
├── auth.rs              — SubsonicAuth extractor (query/form params)
├── extract.rs           — QueryOrForm<T> extractor
├── response.rs          — SubsonicResponse<T> wrapper
├── error.rs             — SubsonicError + ErrorCode
├── nexus.rs             — ID engine, canonical queries, type converters  ← core
├── db/
│   ├── mod.rs           — bb8 pool builder, migration runner
│   ├── models.rs        — Diesel Queryable/Insertable model types
│   └── schema.rs        — Diesel table! macros (auto-generated)
├── scanner/
│   ├── mod.rs           — scan_server(), upsert helpers
│   ├── template.rs      — aggregation-key template engine
│   └── tests.rs
└── handlers/
    ├── admin.rs         — HTML dashboard + scan trigger (partially implemented)
    ├── system.rs        — ping, getLicense, extensions (implemented)
    ├── browsing.rs      — getArtists/Artist/Album/Song, getGenres, …
    ├── searching.rs     — search, search2, search3
    ├── lists.rs         — getAlbumList2, getRandomSongs, …
    ├── media_retrieval.rs — stream, download, getCoverArt, …
    ├── media_annotation.rs — star, unstar, scrobble, …
    ├── playlists.rs     — playlists + play queues
    ├── internet_radio.rs
    ├── podcast.rs
    ├── scanning.rs      — getScanStatus, startScan
    ├── user_management.rs
    ├── bookmarks.rs
    ├── sharing.rs
    ├── chat.rs
    ├── jukebox.rs
    ├── transcoding.rs
    └── advanced.rs
```

## Configuration (`nexus.toml`)

```toml
[[server]]
name = "primary"
url  = "https://navidrome.example.com"
username = "alice"
password = "secret"
priority = 0           # lower = higher priority (canonical source)

[server.matching]
artist = "{music_brainz_id:-{name|lowercase|trim}}"
album  = "{music_brainz_id:-{artist_key}:{name|lowercase|trim}}"

[nexus]
entity_id_template = "{upstream_id}"   # stable IDs based on upstream IDs
proxy = false                           # HTTP 302 redirect for media (default)
write_target = "primary"               # server for write operations (optional)
```

## Running

```bash
cargo run          # starts on 0.0.0.0:3000
curl 'http://localhost:3000/rest/ping?u=x&t=y&s=z&v=1.16.1&c=test&f=json'
# POST /admin/scan  — trigger a background library scan
```
