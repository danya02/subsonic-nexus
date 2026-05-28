use opensubsonic::data::Bookmark;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::error::SubsonicError;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BookmarksBody {
    pub bookmark: Vec<Bookmark>,
}

#[derive(Serialize)]
pub struct BookmarksResponse {
    pub bookmarks: BookmarksBody,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET/POST /rest/getBookmarks — no extra parameters
pub async fn get_bookmarks(_auth: SubsonicAuth) -> SubsonicResponse<BookmarksResponse> {
    BookmarksResponse {
        bookmarks: BookmarksBody { bookmark: vec![] },
    }
    .into()
}

// --- createBookmark ---

#[derive(Deserialize)]
pub struct CreateBookmarkParams {
    pub id: String,
    pub position: i64,
    pub comment: Option<String>,
}

/// GET/POST /rest/createBookmark
pub async fn create_bookmark(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<CreateBookmarkParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Bookmarks are not supported"))
}

// --- deleteBookmark ---

#[derive(Deserialize)]
pub struct DeleteBookmarkParams {
    pub id: String,
}

/// GET/POST /rest/deleteBookmark
pub async fn delete_bookmark(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<DeleteBookmarkParams>,
) -> Result<SubsonicResponse<Empty>, SubsonicError> {
    Err(SubsonicError::not_authorized("Bookmarks are not supported"))
}
