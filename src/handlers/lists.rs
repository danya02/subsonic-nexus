use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getAlbumList
pub async fn get_album_list(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getAlbumList2
pub async fn get_album_list2(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getRandomSongs
pub async fn get_random_songs(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getSongsByGenre
pub async fn get_songs_by_genre(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getNowPlaying
pub async fn get_now_playing(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getStarred
pub async fn get_starred(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getStarred2
pub async fn get_starred2(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getSimilarSongs
pub async fn get_similar_songs(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getSimilarSongs2
pub async fn get_similar_songs2(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/getTopSongs
pub async fn get_top_songs(_auth: SubsonicAuth) -> Response {
    todo!()
}
