use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getBookmarks
pub async fn get_bookmarks(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/createBookmark
pub async fn create_bookmark(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/deleteBookmark
pub async fn delete_bookmark(_auth: SubsonicAuth) -> Response {
    todo!()
}
