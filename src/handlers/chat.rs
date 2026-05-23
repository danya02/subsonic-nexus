use axum::response::Response;

use crate::auth::SubsonicAuth;

/// GET/POST /rest/getChatMessages
pub async fn get_chat_messages(_auth: SubsonicAuth) -> Response {
    todo!()
}

/// GET/POST /rest/addChatMessage
pub async fn add_chat_message(_auth: SubsonicAuth) -> Response {
    todo!()
}
