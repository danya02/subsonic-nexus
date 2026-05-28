//! Unit tests for scanner aggregation logic.
//!
//! All tests use an in-memory SQLite database with the full schema applied —
//! no real upstream server is needed.

use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel_migrations::MigrationHarness;

use crate::db::MIGRATIONS;
use crate::db::models::*;
use crate::db::schema::*;
use crate::scanner::template::Template;
use crate::scanner::{delete_server_data, ensure_server_row};

// ── Test DB setup ─────────────────────────────────────────────────────────────

fn test_conn() -> SqliteConnection {
    let mut conn =
        SqliteConnection::establish(":memory:").expect("Failed to open in-memory SQLite");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to apply migrations");
    // Enable foreign keys in SQLite
    diesel::sql_query("PRAGMA foreign_keys = ON")
        .execute(&mut conn)
        .ok();
    conn
}

// ── Fixture helpers ───────────────────────────────────────────────────────────

fn make_server(conn: &mut SqliteConnection, name: &str, priority: i32) -> i32 {
    diesel::insert_into(upstream_servers::table)
        .values(NewUpstreamServer {
            name,
            url: &format!("https://{name}.example.com"),
            priority,
            last_scanned_at: None,
            artist_key_template: Some("{name|lowercase|trim}"),
            album_key_template: Some("{artist_key}:{name|lowercase|trim}"),
        })
        .execute(conn)
        .unwrap();
    upstream_servers::table
        .order(upstream_servers::id.desc())
        .select(upstream_servers::id)
        .first(conn)
        .unwrap()
}

fn make_artist(
    conn: &mut SqliteConnection,
    server_id: i32,
    upstream_id: &str,
    agg_key: &str,
    name: &str,
) -> i32 {
    diesel::insert_into(artists::table)
        .values(NewArtist {
            server_id,
            upstream_id,
            aggregation_key: agg_key,
            name,
            metadata_json: "{}",
        })
        .execute(conn)
        .unwrap();
    artists::table
        .order(artists::id.desc())
        .select(artists::id)
        .first(conn)
        .unwrap()
}

fn make_album(
    conn: &mut SqliteConnection,
    server_id: i32,
    upstream_id: &str,
    agg_key: &str,
    artist_id: Option<i32>,
    name: &str,
    year: Option<i32>,
    genre: Option<&str>,
) -> i32 {
    diesel::insert_into(albums::table)
        .values(NewAlbum {
            server_id,
            upstream_id,
            aggregation_key: agg_key,
            artist_id,
            name,
            year,
            genre,
            created_at: None,
            play_count: None,
            played_at: None,
            user_rating: None,
            metadata_json: "{}",
        })
        .execute(conn)
        .unwrap();
    albums::table
        .order(albums::id.desc())
        .select(albums::id)
        .first(conn)
        .unwrap()
}

fn make_song(
    conn: &mut SqliteConnection,
    server_id: i32,
    upstream_id: &str,
    album_id: Option<i32>,
    artist_id: Option<i32>,
    title: &str,
) -> i32 {
    diesel::insert_into(songs::table)
        .values(NewSong {
            server_id,
            upstream_id,
            album_id,
            artist_id,
            title,
            year: None,
            genre: None,
            metadata_json: "{}",
        })
        .execute(conn)
        .unwrap();
    songs::table
        .order(songs::id.desc())
        .select(songs::id)
        .first(conn)
        .unwrap()
}

// ── Template tests ────────────────────────────────────────────────────────────

#[test]
fn test_template_mbid_present() {
    let t = Template::parse("{music_brainz_id:-{name|lowercase|trim}}");
    let key = t.eval(&|f| match f {
        "music_brainz_id" => Some("mbid-abc".to_owned()),
        "name" => Some("Daft Punk".to_owned()),
        _ => None,
    });
    assert_eq!(key, "mbid-abc", "should use MusicBrainz ID when present");
}

#[test]
fn test_template_mbid_absent_fallback() {
    let t = Template::parse("{music_brainz_id:-{name|lowercase|trim}}");
    let key = t.eval(&|f| match f {
        "name" => Some("Daft Punk".to_owned()),
        _ => None,
    });
    assert_eq!(key, "daft punk", "should fall back to normalized name");
}

#[test]
fn test_template_different_case_same_key() {
    let t = Template::parse("{name|lowercase|trim}");
    let k1 = t.eval(&|f| {
        if f == "name" {
            Some("Daft Punk".to_owned())
        } else {
            None
        }
    });
    let k2 = t.eval(&|f| {
        if f == "name" {
            Some("daft punk".to_owned())
        } else {
            None
        }
    });
    assert_eq!(
        k1, k2,
        "case variants should produce the same aggregation key"
    );
}

// ── Aggregation logic tests ───────────────────────────────────────────────────

/// Two servers each have "Daft Punk". `getArtists` should return one entry
/// (the primary server's row, with the lowest priority number).
#[test]
fn test_artist_aggregation() {
    let mut conn = test_conn();
    let s1 = make_server(&mut conn, "server1", 0); // primary
    let s2 = make_server(&mut conn, "server2", 1); // secondary

    let _a1 = make_artist(&mut conn, s1, "uid-s1-dp", "daft punk", "Daft Punk");
    let _a2 = make_artist(&mut conn, s2, "uid-s2-dp", "daft punk", "daft punk");

    // Simulate getArtists: per agg_key group, pick row from lowest-priority server.
    let primary: Artist = artists::table
        .inner_join(upstream_servers::table)
        .filter(artists::aggregation_key.eq("daft punk"))
        .order(upstream_servers::priority.asc())
        .select(Artist::as_select())
        .first(&mut conn)
        .unwrap();

    assert_eq!(primary.server_id, s1);
    assert_eq!(primary.upstream_id, "uid-s1-dp");
}

/// getAlbum resolution: given an upstream_id that matches both servers' album
/// rows for the same logical album, the primary server's row should win.
#[test]
fn test_album_primary_server_wins() {
    let mut conn = test_conn();
    let s1 = make_server(&mut conn, "server1", 0);
    let s2 = make_server(&mut conn, "server2", 1);

    let a1 = make_artist(&mut conn, s1, "dp-s1", "daft punk", "Daft Punk");
    let a2 = make_artist(&mut conn, s2, "dp-s2", "daft punk", "Daft Punk");

    let alb1 = make_album(
        &mut conn,
        s1,
        "ram-s1",
        "daft punk:random access memories",
        Some(a1),
        "Random Access Memories",
        Some(2013),
        None,
    );
    let alb2 = make_album(
        &mut conn,
        s2,
        "ram-s2",
        "daft punk:random access memories",
        Some(a2),
        "Random Access Memories",
        Some(2013),
        None,
    );

    // Simulate getAlbum("ram-s1"):
    // 1. Find row with upstream_id = "ram-s1" → alb1 (s1)
    // 2. Get its aggregation_key
    // 3. Find all albums with that key, pick the one from the lowest-priority server.
    let hit: Album = albums::table
        .filter(albums::upstream_id.eq("ram-s1"))
        .first(&mut conn)
        .unwrap();
    let agg_key = &hit.aggregation_key;

    let primary_album: Album = albums::table
        .inner_join(upstream_servers::table)
        .filter(albums::aggregation_key.eq(agg_key))
        .order(upstream_servers::priority.asc())
        .select(Album::as_select())
        .first(&mut conn)
        .unwrap();

    assert_eq!(
        primary_album.id, alb1,
        "primary album should be from server1"
    );
    // Server2's row still exists
    assert_ne!(alb1, alb2);
}

/// Songs are returned from the primary server's album instance.
#[test]
fn test_songs_from_primary_album() {
    let mut conn = test_conn();
    let s1 = make_server(&mut conn, "server1", 0);
    let s2 = make_server(&mut conn, "server2", 1);

    let a1 = make_artist(&mut conn, s1, "dp-s1", "daft punk", "Daft Punk");
    let a2 = make_artist(&mut conn, s2, "dp-s2", "daft punk", "Daft Punk");
    let alb1 = make_album(
        &mut conn,
        s1,
        "ram-s1",
        "daft punk:ram",
        Some(a1),
        "RAM",
        None,
        None,
    );
    let alb2 = make_album(
        &mut conn,
        s2,
        "ram-s2",
        "daft punk:ram",
        Some(a2),
        "RAM",
        None,
        None,
    );

    let _s1_song1 = make_song(
        &mut conn,
        s1,
        "song-1-s1",
        Some(alb1),
        Some(a1),
        "Get Lucky",
    );
    let _s1_song2 = make_song(
        &mut conn,
        s1,
        "song-2-s1",
        Some(alb1),
        Some(a1),
        "Instant Crush",
    );
    let _s2_song1 = make_song(
        &mut conn,
        s2,
        "song-1-s2",
        Some(alb2),
        Some(a2),
        "Get Lucky",
    );

    // After resolving primary album = alb1, fetch its songs.
    let song_titles: Vec<String> = songs::table
        .filter(songs::album_id.eq(alb1))
        .select(songs::title)
        .load(&mut conn)
        .unwrap();

    assert_eq!(
        song_titles.len(),
        2,
        "should have exactly 2 songs from server1's album"
    );
    assert!(song_titles.contains(&"Get Lucky".to_owned()));
    assert!(song_titles.contains(&"Instant Crush".to_owned()));
}

/// Artist album union: same artist on two servers, each with some unique albums.
/// getArtist should return all unique album aggregation_keys combined.
#[test]
fn test_artist_album_union() {
    let mut conn = test_conn();
    let s1 = make_server(&mut conn, "server1", 0);
    let s2 = make_server(&mut conn, "server2", 1);

    let a1 = make_artist(&mut conn, s1, "dp-s1", "daft punk", "Daft Punk");
    let a2 = make_artist(&mut conn, s2, "dp-s2", "daft punk", "Daft Punk");

    // Server1 has 3 albums
    make_album(
        &mut conn,
        s1,
        "alb-s1-1",
        "daft punk:homework",
        Some(a1),
        "Homework",
        Some(1997),
        None,
    );
    make_album(
        &mut conn,
        s1,
        "alb-s1-2",
        "daft punk:discovery",
        Some(a1),
        "Discovery",
        Some(2001),
        None,
    );
    make_album(
        &mut conn,
        s1,
        "alb-s1-3",
        "daft punk:ram",
        Some(a1),
        "RAM",
        Some(2013),
        None,
    );

    // Server2 has 2 albums (overlapping discovery + a unique one)
    make_album(
        &mut conn,
        s2,
        "alb-s2-1",
        "daft punk:discovery",
        Some(a2),
        "Discovery",
        Some(2001),
        None,
    );
    make_album(
        &mut conn,
        s2,
        "alb-s2-2",
        "daft punk:alive 2007",
        Some(a2),
        "Alive 2007",
        Some(2007),
        None,
    );

    // Find all artist rows with agg_key "daft punk", collect all album rows,
    // group by album agg_key — should yield 4 distinct albums.
    let artist_ids: Vec<i32> = artists::table
        .filter(artists::aggregation_key.eq("daft punk"))
        .select(artists::id)
        .load(&mut conn)
        .unwrap();

    let mut album_keys: Vec<String> = albums::table
        .filter(albums::artist_id.eq_any(&artist_ids))
        .select(albums::aggregation_key)
        .load(&mut conn)
        .unwrap();
    album_keys.sort();
    album_keys.dedup();

    assert_eq!(
        album_keys.len(),
        4,
        "should have 4 distinct album aggregation keys"
    );
    assert!(album_keys.contains(&"daft punk:alive 2007".to_owned()));
    assert!(album_keys.contains(&"daft punk:discovery".to_owned()));
    assert!(album_keys.contains(&"daft punk:homework".to_owned()));
    assert!(album_keys.contains(&"daft punk:ram".to_owned()));
}

/// Stream / coverArt routing: song's server_id + upstream_id should match
/// the server it came from so we can proxy the request correctly.
#[test]
fn test_song_routing() {
    let mut conn = test_conn();
    let s1 = make_server(&mut conn, "server1", 0);

    let a1 = make_artist(&mut conn, s1, "dp-s1", "daft punk", "Daft Punk");
    let alb1 = make_album(
        &mut conn,
        s1,
        "ram-s1",
        "daft punk:ram",
        Some(a1),
        "RAM",
        None,
        None,
    );
    let _ = make_song(
        &mut conn,
        s1,
        "get-lucky-s1",
        Some(alb1),
        Some(a1),
        "Get Lucky",
    );

    // Simulate stream("get-lucky-s1"):
    let (found_server_id, found_upstream_id): (i32, String) = songs::table
        .filter(songs::upstream_id.eq("get-lucky-s1"))
        .select((songs::server_id, songs::upstream_id))
        .first(&mut conn)
        .unwrap();

    assert_eq!(found_server_id, s1);
    assert_eq!(found_upstream_id, "get-lucky-s1");

    // Verify we can look up the server URL from that server_id
    let server_url: String = upstream_servers::table
        .find(found_server_id)
        .select(upstream_servers::url)
        .first(&mut conn)
        .unwrap();
    assert!(server_url.contains("server1"));
}

/// Template change detection: ensure_server_row should mark rescan_needed=true
/// when templates differ.
#[test]
fn test_template_change_triggers_rescan() {
    let mut conn = test_conn();

    let cfg1 = crate::config::ServerConfig {
        name: "myserver".to_owned(),
        url: "https://myserver.example.com".to_owned(),
        username: "u".to_owned(),
        password: "p".to_owned(),
        priority: 0,
        matching: crate::config::MatchingConfig {
            artist: "{name|lowercase}".to_owned(),
            album: "{artist_key}:{name|lowercase}".to_owned(),
        },
    };

    let (server_id, needs_scan) = ensure_server_row(&mut conn, &cfg1).unwrap();
    assert!(needs_scan, "first run should always need a scan");

    // Insert some data to verify it gets wiped on template change.
    make_artist(&mut conn, server_id, "a1", "some key", "Artist 1");
    let count: i64 = artists::table.count().get_result(&mut conn).unwrap();
    assert_eq!(count, 1);

    // Same config → no rescan needed.
    let (_, needs_scan2) = ensure_server_row(&mut conn, &cfg1).unwrap();
    assert!(!needs_scan2, "same templates should not trigger a rescan");

    // Changed artist template → rescan needed + data wiped.
    let cfg2 = crate::config::ServerConfig {
        matching: crate::config::MatchingConfig {
            artist: "{music_brainz_id:-{name|lowercase}}".to_owned(),
            ..cfg1.matching.clone()
        },
        ..cfg1.clone()
    };
    let (_, needs_scan3) = ensure_server_row(&mut conn, &cfg2).unwrap();
    assert!(
        needs_scan3,
        "changed artist template should trigger a rescan"
    );

    let count_after: i64 = artists::table.count().get_result(&mut conn).unwrap();
    assert_eq!(count_after, 0, "stale artist data should have been deleted");
}

/// delete_server_data should remove all child rows for the given server.
#[test]
fn test_delete_server_data() {
    let mut conn = test_conn();
    let s1 = make_server(&mut conn, "server1", 0);
    let s2 = make_server(&mut conn, "server2", 1);

    let a1 = make_artist(&mut conn, s1, "a1", "artist one", "Artist One");
    let a2 = make_artist(&mut conn, s2, "a2", "artist two", "Artist Two");
    let alb1 = make_album(
        &mut conn,
        s1,
        "alb1",
        "a1:alb",
        Some(a1),
        "Album 1",
        None,
        None,
    );
    let _ = make_album(
        &mut conn,
        s2,
        "alb2",
        "a2:alb",
        Some(a2),
        "Album 2",
        None,
        None,
    );
    make_song(&mut conn, s1, "song1", Some(alb1), Some(a1), "Song 1");

    delete_server_data(&mut conn, s1).unwrap();

    let artist_count: i64 = artists::table.count().get_result(&mut conn).unwrap();
    let album_count: i64 = albums::table.count().get_result(&mut conn).unwrap();
    let song_count: i64 = songs::table.count().get_result(&mut conn).unwrap();

    assert_eq!(artist_count, 1, "server2's artist should remain");
    assert_eq!(album_count, 1, "server2's album should remain");
    assert_eq!(song_count, 0, "server1's song should be gone");
}
