use axum::{
    Router,
    routing::{get, post, MethodRouter},
};

use crate::handlers::{
    advanced, bookmarks, browsing, chat, internet_radio, jukebox, lists, media_annotation,
    media_retrieval, playlists, podcast, scanning, searching, sharing, system, transcoding,
    user_management,
};

/// Register a route that accepts both GET and POST with the same handler.
fn get_post<H, T, S>(handler: H) -> MethodRouter<S>
where
    H: axum::handler::Handler<T, S> + Clone,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    get(handler.clone()).post(handler)
}

pub fn build_router() -> Router {
    Router::new()
        // ── System ──────────────────────────────────────────────────────────
        .route("/rest/ping",                        get_post(system::ping))
        .route("/rest/getLicense",                  get_post(system::get_license))
        .route("/rest/getOpenSubsonicExtensions",   get_post(system::get_open_subsonic_extensions))
        .route("/rest/tokenInfo",                   get_post(system::token_info))

        // ── Browsing ─────────────────────────────────────────────────────────
        .route("/rest/getMusicFolders",             get_post(browsing::get_music_folders))
        .route("/rest/getIndexes",                  get_post(browsing::get_indexes))
        .route("/rest/getMusicDirectory",           get_post(browsing::get_music_directory))
        .route("/rest/getGenres",                   get_post(browsing::get_genres))
        .route("/rest/getArtists",                  get_post(browsing::get_artists))
        .route("/rest/getArtist",                   get_post(browsing::get_artist))
        .route("/rest/getAlbum",                    get_post(browsing::get_album))
        .route("/rest/getSong",                     get_post(browsing::get_song))
        .route("/rest/getVideos",                   get_post(browsing::get_videos))
        .route("/rest/getVideoInfo",                get_post(browsing::get_video_info))
        .route("/rest/getArtistInfo",               get_post(browsing::get_artist_info))
        .route("/rest/getArtistInfo2",              get_post(browsing::get_artist_info2))
        .route("/rest/getAlbumInfo",                get_post(browsing::get_album_info))
        .route("/rest/getAlbumInfo2",               get_post(browsing::get_album_info2))

        // ── Lists ────────────────────────────────────────────────────────────
        .route("/rest/getAlbumList",                get_post(lists::get_album_list))
        .route("/rest/getAlbumList2",               get_post(lists::get_album_list2))
        .route("/rest/getRandomSongs",              get_post(lists::get_random_songs))
        .route("/rest/getSongsByGenre",             get_post(lists::get_songs_by_genre))
        .route("/rest/getNowPlaying",               get_post(lists::get_now_playing))
        .route("/rest/getStarred",                  get_post(lists::get_starred))
        .route("/rest/getStarred2",                 get_post(lists::get_starred2))
        .route("/rest/getSimilarSongs",             get_post(lists::get_similar_songs))
        .route("/rest/getSimilarSongs2",            get_post(lists::get_similar_songs2))
        .route("/rest/getTopSongs",                 get_post(lists::get_top_songs))

        // ── Searching ────────────────────────────────────────────────────────
        .route("/rest/search",                      get_post(searching::search))
        .route("/rest/search2",                     get_post(searching::search2))
        .route("/rest/search3",                     get_post(searching::search3))

        // ── Playlists ────────────────────────────────────────────────────────
        .route("/rest/getPlaylists",                get_post(playlists::get_playlists))
        .route("/rest/getPlaylist",                 get_post(playlists::get_playlist))
        .route("/rest/createPlaylist",              get_post(playlists::create_playlist))
        .route("/rest/updatePlaylist",              get_post(playlists::update_playlist))
        .route("/rest/deletePlaylist",              get_post(playlists::delete_playlist))
        .route("/rest/getPlayQueue",                get_post(playlists::get_play_queue))
        .route("/rest/savePlayQueue",               get_post(playlists::save_play_queue))
        .route("/rest/getPlayQueueByIndex",         get_post(playlists::get_play_queue_by_index))
        .route("/rest/savePlayQueueByIndex",        get_post(playlists::save_play_queue_by_index))

        // ── Media Retrieval ──────────────────────────────────────────────────
        .route("/rest/stream",                      get_post(media_retrieval::stream))
        .route("/rest/download",                    get_post(media_retrieval::download))
        .route("/rest/getCoverArt",                 get_post(media_retrieval::get_cover_art))
        .route("/rest/getLyrics",                   get_post(media_retrieval::get_lyrics))
        .route("/rest/getLyricsBySongId",           get_post(media_retrieval::get_lyrics_by_song_id))
        .route("/rest/getAvatar",                   get_post(media_retrieval::get_avatar))
        .route("/rest/getCaptions",                 get_post(media_retrieval::get_captions))
        .route("/rest/hls.m3u8",                    get_post(media_retrieval::hls))

        // ── Media Annotation ─────────────────────────────────────────────────
        .route("/rest/star",                        get_post(media_annotation::star))
        .route("/rest/unstar",                      get_post(media_annotation::unstar))
        .route("/rest/setRating",                   get_post(media_annotation::set_rating))
        .route("/rest/scrobble",                    get_post(media_annotation::scrobble))
        .route("/rest/reportPlayback",              get_post(media_annotation::report_playback))

        // ── Sharing ──────────────────────────────────────────────────────────
        .route("/rest/getShares",                   get_post(sharing::get_shares))
        .route("/rest/createShare",                 get_post(sharing::create_share))
        .route("/rest/updateShare",                 get_post(sharing::update_share))
        .route("/rest/deleteShare",                 get_post(sharing::delete_share))

        // ── Podcast ──────────────────────────────────────────────────────────
        .route("/rest/getPodcasts",                 get_post(podcast::get_podcasts))
        .route("/rest/getNewestPodcasts",           get_post(podcast::get_newest_podcasts))
        .route("/rest/getPodcastEpisode",           get_post(podcast::get_podcast_episode))
        .route("/rest/createPodcastChannel",        get_post(podcast::create_podcast_channel))
        .route("/rest/deletePodcastChannel",        get_post(podcast::delete_podcast_channel))
        .route("/rest/deletePodcastEpisode",        get_post(podcast::delete_podcast_episode))
        .route("/rest/downloadPodcastEpisode",      get_post(podcast::download_podcast_episode))
        .route("/rest/refreshPodcasts",             get_post(podcast::refresh_podcasts))

        // ── Jukebox ──────────────────────────────────────────────────────────
        .route("/rest/jukeboxControl",              get_post(jukebox::jukebox_control))

        // ── Internet Radio ───────────────────────────────────────────────────
        .route("/rest/getInternetRadioStations",    get_post(internet_radio::get_internet_radio_stations))
        .route("/rest/createInternetRadioStation",  get_post(internet_radio::create_internet_radio_station))
        .route("/rest/updateInternetRadioStation",  get_post(internet_radio::update_internet_radio_station))
        .route("/rest/deleteInternetRadioStation",  get_post(internet_radio::delete_internet_radio_station))

        // ── Chat ─────────────────────────────────────────────────────────────
        .route("/rest/getChatMessages",             get_post(chat::get_chat_messages))
        .route("/rest/addChatMessage",              get_post(chat::add_chat_message))

        // ── User Management ──────────────────────────────────────────────────
        .route("/rest/getUser",                     get_post(user_management::get_user))
        .route("/rest/getUsers",                    get_post(user_management::get_users))
        .route("/rest/createUser",                  get_post(user_management::create_user))
        .route("/rest/updateUser",                  get_post(user_management::update_user))
        .route("/rest/deleteUser",                  get_post(user_management::delete_user))
        .route("/rest/changePassword",              get_post(user_management::change_password))

        // ── Bookmarks ────────────────────────────────────────────────────────
        .route("/rest/getBookmarks",                get_post(bookmarks::get_bookmarks))
        .route("/rest/createBookmark",              get_post(bookmarks::create_bookmark))
        .route("/rest/deleteBookmark",              get_post(bookmarks::delete_bookmark))

        // ── Library Scanning ─────────────────────────────────────────────────
        .route("/rest/getScanStatus",               get_post(scanning::get_scan_status))
        .route("/rest/startScan",                   get_post(scanning::start_scan))

        // ── Transcoding ──────────────────────────────────────────────────────
        // getTranscodeDecision: POST only per spec
        .route("/rest/getTranscodeDecision",        post(transcoding::get_transcode_decision))
        // getTranscodeStream: GET only per spec
        .route("/rest/getTranscodeStream",          get(transcoding::get_transcode_stream))

        // ── Advanced / OpenSubsonic Extensions ───────────────────────────────
        .route("/rest/findSonicPath",               get_post(advanced::find_sonic_path))
        .route("/rest/getSonicSimilarTracks",       get_post(advanced::get_sonic_similar_tracks))
}
