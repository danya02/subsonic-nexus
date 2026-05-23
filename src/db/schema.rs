// @generated automatically by Diesel CLI.

diesel::table! {
    albums (id) {
        id -> Integer,
        server_id -> Integer,
        upstream_id -> Text,
        aggregation_key -> Text,
        artist_id -> Nullable<Integer>,
        name -> Text,
        year -> Nullable<Integer>,
        genre -> Nullable<Text>,
        created_at -> Nullable<Text>,
        play_count -> Nullable<Integer>,
        played_at -> Nullable<Text>,
        user_rating -> Nullable<Integer>,
        metadata_json -> Text,
    }
}

diesel::table! {
    artists (id) {
        id -> Integer,
        server_id -> Integer,
        upstream_id -> Text,
        aggregation_key -> Text,
        name -> Text,
        metadata_json -> Text,
    }
}

diesel::table! {
    internet_radio_stations (id) {
        id -> Integer,
        server_id -> Integer,
        upstream_id -> Text,
        name -> Text,
        stream_url -> Text,
        metadata_json -> Text,
    }
}

diesel::table! {
    podcast_channels (id) {
        id -> Integer,
        server_id -> Integer,
        upstream_id -> Text,
        title -> Nullable<Text>,
        metadata_json -> Text,
    }
}

diesel::table! {
    podcast_episodes (id) {
        id -> Integer,
        server_id -> Integer,
        upstream_id -> Text,
        channel_id -> Nullable<Integer>,
        title -> Nullable<Text>,
        publish_date -> Nullable<Text>,
        metadata_json -> Text,
    }
}

diesel::table! {
    songs (id) {
        id -> Integer,
        server_id -> Integer,
        upstream_id -> Text,
        album_id -> Nullable<Integer>,
        artist_id -> Nullable<Integer>,
        title -> Text,
        year -> Nullable<Integer>,
        genre -> Nullable<Text>,
        metadata_json -> Text,
    }
}

diesel::table! {
    upstream_servers (id) {
        id -> Integer,
        name -> Text,
        url -> Text,
        priority -> Integer,
        last_scanned_at -> Nullable<Text>,
        artist_key_template -> Nullable<Text>,
        album_key_template -> Nullable<Text>,
    }
}

diesel::joinable!(albums -> artists (artist_id));
diesel::joinable!(albums -> upstream_servers (server_id));
diesel::joinable!(artists -> upstream_servers (server_id));
diesel::joinable!(internet_radio_stations -> upstream_servers (server_id));
diesel::joinable!(podcast_channels -> upstream_servers (server_id));
diesel::joinable!(podcast_episodes -> podcast_channels (channel_id));
diesel::joinable!(podcast_episodes -> upstream_servers (server_id));
diesel::joinable!(songs -> albums (album_id));
diesel::joinable!(songs -> artists (artist_id));
diesel::joinable!(songs -> upstream_servers (server_id));

diesel::allow_tables_to_appear_in_same_query!(
    albums,
    artists,
    internet_radio_stations,
    podcast_channels,
    podcast_episodes,
    songs,
    upstream_servers,
);
