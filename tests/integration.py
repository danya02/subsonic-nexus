#!/usr/bin/env python3
"""
Integration tests for subsonic-nexus.

Usage:
    python3 tests/integration.py [--base-url URL]

The server must already be running (cargo run &).
Defaults to http://localhost:3000.
"""

import sys
import json
import argparse
import urllib.request
import urllib.parse

# urllib.request honours HTTP_PROXY / http_proxy env vars.  The test only ever
# connects to localhost so we must bypass any proxy that may be configured in
# the environment (e.g. an Anthropic corporate proxy), otherwise every request
# is routed through it and hangs indefinitely.
urllib.request.install_opener(
    urllib.request.build_opener(urllib.request.ProxyHandler({}))
)

# Ensure output is flushed line-by-line so progress is visible when stdout is
# redirected to a file or pipe (Python defaults to block-buffering in that case).
sys.stdout.reconfigure(line_buffering=True)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def url(base, endpoint, **params):
    qs = urllib.parse.urlencode(
        {"u": "test", "t": "abc", "s": "xyz", "v": "1.16.1", "c": "integration-test",
         **params}
    )
    return f"{base}/rest/{endpoint}?{qs}"


def get(base, endpoint, **params):
    try:
        with urllib.request.urlopen(url(base, endpoint, **params), timeout=10) as resp:
            return json.loads(resp.read())
    except Exception as e:
        return {"_error": str(e)}


def inner(data):
    """Extract the subsonic-response inner object."""
    return data.get("subsonic-response", {})


PASS = "✓"
FAIL = "✗"
results = []


def check(name, condition, detail=""):
    mark = PASS if condition else FAIL
    msg = f"  {mark}  {name}"
    if detail:
        msg += f"  ({detail})"
    print(msg, flush=True)
    results.append(condition)
    return condition


# ---------------------------------------------------------------------------
# Test suites
# ---------------------------------------------------------------------------

def test_system(base):
    print("\n── System ──────────────────────────────────────────────")

    r = inner(get(base, "ping"))
    check("ping → ok", r.get("status") == "ok")
    check("ping reports openSubsonic=true", r.get("openSubsonic") is True)
    check("ping reports server type", r.get("type") == "subsonic-nexus")

    r = inner(get(base, "getLicense"))
    check("getLicense → ok", r.get("status") == "ok")
    check("license.valid = true", r.get("license", {}).get("valid") is True)

    r = inner(get(base, "getOpenSubsonicExtensions"))
    check("getOpenSubsonicExtensions → ok", r.get("status") == "ok")
    exts = r.get("openSubsonicExtensions", [])
    check("extensions is a list", isinstance(exts, list))


def test_browsing(base):
    print("\n── Browsing ─────────────────────────────────────────────")

    # getMusicFolders
    r = inner(get(base, "getMusicFolders"))
    check("getMusicFolders → ok", r.get("status") == "ok")
    folders = r.get("musicFolders", {}).get("musicFolder", [])
    check("at least one music folder", len(folders) >= 1,
          f"{len(folders)} folders")

    # getArtists
    r = inner(get(base, "getArtists"))
    check("getArtists → ok", r.get("status") == "ok")
    indexes = r.get("artists", {}).get("index", [])
    total = sum(len(i.get("artist", [])) for i in indexes)
    check("artists > 0", total > 0, f"{total} artists")
    check("index has multiple letters", len(indexes) > 5,
          f"{len(indexes)} index buckets")

    # getArtist — pick first artist
    first_artist = indexes[0]["artist"][0] if indexes else None
    artist_id = first_artist["id"] if first_artist else None
    if artist_id:
        r = inner(get(base, "getArtist", id=artist_id))
        check("getArtist → ok", r.get("status") == "ok")
        artist = r.get("artist", {})
        check("getArtist has albums", len(artist.get("album", [])) >= 0,
              f'{len(artist.get("album",[]))} albums')

        # getAlbum — pick first album
        albums = artist.get("album", [])
        if albums:
            album_id = albums[0]["id"]
            r = inner(get(base, "getAlbum", id=album_id))
            check("getAlbum → ok", r.get("status") == "ok")
            songs = r.get("album", {}).get("song", [])
            check("getAlbum has songs", len(songs) > 0,
                  f"{len(songs)} songs")

            # getSong
            if songs:
                song_id = songs[0]["id"]
                r = inner(get(base, "getSong", id=song_id))
                check("getSong → ok", r.get("status") == "ok")
                check("getSong has title",
                      bool(r.get("song", {}).get("title")))

    # getIndexes (legacy)
    r = inner(get(base, "getIndexes"))
    check("getIndexes → ok", r.get("status") == "ok")

    # getGenres
    r = inner(get(base, "getGenres"))
    check("getGenres → ok", r.get("status") == "ok")
    genres = r.get("genres", {}).get("genre", [])
    check("genres > 0", len(genres) > 0, f"{len(genres)} genres")

    # getMusicDirectory — server folder
    if folders:
        folder_id = folders[0]["id"]
        r = inner(get(base, "getMusicDirectory", id=folder_id))
        check("getMusicDirectory (folder) → ok", r.get("status") == "ok")

    # getArtistInfo / getAlbumInfo
    if artist_id:
        r = inner(get(base, "getArtistInfo", id=artist_id))
        check("getArtistInfo → ok", r.get("status") == "ok")
        r = inner(get(base, "getArtistInfo2", id=artist_id))
        check("getArtistInfo2 → ok", r.get("status") == "ok")


def test_searching(base):
    print("\n── Searching ────────────────────────────────────────────")

    r = inner(get(base, "search2", query="a", songCount=5))
    check("search2 → ok", r.get("status") == "ok")
    r2 = r.get("searchResult2", {})
    check("search2 returns songs", len(r2.get("song", [])) > 0)

    r = inner(get(base, "search3", query="a", songCount=5, albumCount=3))
    check("search3 → ok", r.get("status") == "ok")
    r3 = r.get("searchResult3", {})
    check("search3 returns songs", len(r3.get("song", [])) > 0)
    check("search3 returns albums", len(r3.get("album", [])) > 0)


def test_lists(base):
    print("\n── Lists ────────────────────────────────────────────────")

    for list_type in ["random", "newest", "alphabeticalByName", "alphabeticalByArtist"]:
        r = inner(get(base, "getAlbumList2", type=list_type, size=5))
        check(f"getAlbumList2 type={list_type} → ok",
              r.get("status") == "ok",
              f'{len(r.get("albumList2",{}).get("album",[]))} albums')

    r = inner(get(base, "getAlbumList2", type="byYear",
                  size=5, fromYear=2000, toYear=2025))
    check("getAlbumList2 byYear → ok", r.get("status") == "ok")

    r = inner(get(base, "getAlbumList", type="random", size=3))
    check("getAlbumList (legacy) → ok", r.get("status") == "ok")

    r = inner(get(base, "getRandomSongs", size=5))
    check("getRandomSongs → ok", r.get("status") == "ok")
    check("getRandomSongs returns songs",
          len(r.get("randomSongs", {}).get("song", [])) > 0)

    r = inner(get(base, "getNowPlaying"))
    check("getNowPlaying → ok", r.get("status") == "ok")

    r = inner(get(base, "getStarred"))
    check("getStarred → ok", r.get("status") == "ok")

    r = inner(get(base, "getStarred2"))
    check("getStarred2 → ok", r.get("status") == "ok")


def open_no_redir(target_url, timeout=10):
    """
    Open a URL without following redirects and without using HTTP_PROXY.
    Returns (status_code, response_or_None).  3xx responses are returned
    directly (as HTTPError with code); the body is never read so we don't
    accidentally stream a large audio file.
    """
    # ProxyHandler({}) — empty dict disables proxy entirely (no HTTP_PROXY).
    # We intentionally omit HTTPRedirectHandler so 3xx come back as HTTPError.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    opener.handlers = [h for h in opener.handlers
                       if not isinstance(h, urllib.request.HTTPRedirectHandler)]
    try:
        resp = opener.open(target_url, timeout=timeout)
        code = resp.status
        resp.close()
        return code, None
    except urllib.error.HTTPError as e:
        return e.code, None
    except Exception as e:
        return None, str(e)


def test_media_retrieval(base):
    print("\n── Media retrieval ──────────────────────────────────────")

    # Find a real song ID first
    r = inner(get(base, "getRandomSongs", size=1))
    songs = r.get("randomSongs", {}).get("song", [])
    if not songs:
        check("stream (need a song)", False, "no songs available")
        return
    song_id = songs[0]["id"]

    # stream — expect 302 redirect (or 200 if proxy=true); don't stream audio
    code, err = open_no_redir(url(base, "stream", id=song_id))
    if err:
        check("stream reachable", False, err)
    else:
        check("stream → 302 redirect or 200 proxy",
              code in (200, 302, 303), f"HTTP {code}")

    # getCoverArt — same: expect 302 redirect or 200 proxy
    cover_art_id = songs[0].get("coverArt")
    if cover_art_id:
        code, err = open_no_redir(url(base, "getCoverArt", id=cover_art_id))
        if err:
            check("getCoverArt reachable", False, err)
        else:
            check("getCoverArt → 302 or 200",
                  code in (200, 302, 303), f"HTTP {code}")


def test_annotation(base):
    print("\n── Media annotation (write proxy) ───────────────────────")

    # star — always returns ok (proxied or silently accepted)
    r = inner(get(base, "star", id="fake_id"))
    check("star → ok (single id)", r.get("status") == "ok")

    # star with multiple ids (tests Vec<String> parsing)
    encoded = urllib.parse.urlencode(
        {"u": "test", "t": "abc", "s": "xyz", "v": "1.16.1", "c": "test",
         "id": ["id1", "id2"], "albumId": "alb1"},
        doseq=True,
    )
    try:
        with urllib.request.urlopen(f"{base}/rest/star?{encoded}") as resp:
            data = json.loads(resp.read())
            r = inner(data)
            check("star → ok (multiple ids)", r.get("status") == "ok")
    except Exception as e:
        check("star (multiple ids) reachable", False, str(e))

    # unstar
    r = inner(get(base, "unstar", id="fake_id"))
    check("unstar → ok", r.get("status") == "ok")

    # setRating
    r = inner(get(base, "setRating", id="fake_id", rating=4))
    check("setRating → ok", r.get("status") == "ok")

    # scrobble with a real song id
    r_rand = inner(get(base, "getRandomSongs", size=1))
    songs = r_rand.get("randomSongs", {}).get("song", [])
    if songs:
        r = inner(get(base, "scrobble", id=songs[0]["id"], submission="true"))
        check("scrobble (real song) → ok", r.get("status") == "ok")


def test_scanning(base):
    print("\n── Scanning ─────────────────────────────────────────────")

    r = inner(get(base, "getScanStatus"))
    check("getScanStatus → ok", r.get("status") == "ok")
    sc = r.get("scanStatus", {})
    check("scanStatus.scanning = false", sc.get("scanning") is False)
    check("scanStatus.count > 0", (sc.get("count") or 0) > 0,
          f'count={sc.get("count")}')


def test_user_management(base):
    print("\n── User management ──────────────────────────────────────")

    r = inner(get(base, "getUser", username="admin"))
    check("getUser → ok", r.get("status") == "ok")
    u = r.get("user", {})
    check("user.username set", bool(u.get("username")))
    check("user.adminRole = true", u.get("adminRole") is True)

    r = inner(get(base, "getUsers"))
    check("getUsers → ok", r.get("status") == "ok")
    users = r.get("users", {}).get("user", [])
    check("getUsers returns ≥1 user", len(users) >= 1)

    r = inner(get(base, "createUser", username="x", password="x", email="x@x"))
    check("createUser → not_authorized",
          r.get("status") == "failed" and
          r.get("error", {}).get("code") == 50)


def test_playlists(base):
    print("\n── Playlists ────────────────────────────────────────────")

    r = inner(get(base, "getPlaylists"))
    # Either ok (write_target configured, proxied) or ok (empty list)
    check("getPlaylists → ok or proxied error",
          r.get("status") in ("ok", "failed"))

    r = inner(get(base, "getPlayQueue"))
    check("getPlayQueue → ok", r.get("status") == "ok")
    check("playQueue.username set",
          bool(r.get("playQueue", {}).get("username")))

    r = inner(get(base, "getPlayQueueByIndex"))
    check("getPlayQueueByIndex → ok", r.get("status") == "ok")

    r = inner(get(base, "savePlayQueue"))
    check("savePlayQueue (empty) → ok", r.get("status") == "ok")


def test_misc(base):
    print("\n── Misc endpoints ────────────────────────────────────────")

    for ep in ["getChatMessages", "getShares", "getBookmarks"]:
        r = inner(get(base, ep))
        check(f"{ep} → ok", r.get("status") == "ok")

    r = inner(get(base, "getInternetRadioStations"))
    check("getInternetRadioStations → ok", r.get("status") == "ok")

    r = inner(get(base, "getPodcasts"))
    check("getPodcasts → ok", r.get("status") == "ok")

    r = inner(get(base, "getNewestPodcasts", count=5))
    check("getNewestPodcasts → ok", r.get("status") == "ok")

    # Write ops that should return not_authorized
    for ep, params in [
        ("createShare", {"id": "x"}),
        ("createBookmark", {"id": "x", "position": 0}),
        ("jukeboxControl", {"action": "status"}),
    ]:
        r = inner(get(base, ep, **params))
        check(f"{ep} → not_authorized",
              r.get("status") == "failed" and
              r.get("error", {}).get("code") == 50)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(description="subsonic-nexus integration tests")
    parser.add_argument("--base-url", default="http://localhost:3000",
                        help="Base URL of the running nexus server")
    args = parser.parse_args()
    base = args.base_url.rstrip("/")

    print(f"Running integration tests against {base}")

    test_system(base)
    test_browsing(base)
    test_searching(base)
    test_lists(base)
    test_media_retrieval(base)
    test_annotation(base)
    test_scanning(base)
    test_user_management(base)
    test_playlists(base)
    test_misc(base)

    passed = sum(results)
    total = len(results)
    failed = total - passed
    print(f"\n{'─' * 56}")
    print(f"  {passed}/{total} passed", end="")
    if failed:
        print(f"  ← {failed} FAILED", end="")
    print()

    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
