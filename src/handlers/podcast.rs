use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getPodcasts
pub async fn get_podcasts(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getNewestPodcasts
pub async fn get_newest_podcasts(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getPodcastEpisode
pub async fn get_podcast_episode(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/createPodcastChannel
pub async fn create_podcast_channel(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deletePodcastChannel
pub async fn delete_podcast_channel(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deletePodcastEpisode
pub async fn delete_podcast_episode(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/downloadPodcastEpisode
pub async fn download_podcast_episode(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/refreshPodcasts
pub async fn refresh_podcasts(_auth: SubsonicAuth) -> Response {
    todo!()
}
