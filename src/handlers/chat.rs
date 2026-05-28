use opensubsonic::data::ChatMessage;
use serde::{Deserialize, Serialize};

use crate::auth::SubsonicAuth;
use crate::extract::QueryOrForm;
use crate::response::{Empty, SubsonicResponse};

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ChatMessagesBody {
    #[serde(rename = "chatMessage")]
    pub chat_message: Vec<ChatMessage>,
}

#[derive(Serialize)]
pub struct ChatMessagesResponse {
    #[serde(rename = "chatMessages")]
    pub chat_messages: ChatMessagesBody,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

// --- getChatMessages ---

#[derive(Deserialize)]
pub struct GetChatMessagesParams {
    pub since: Option<i64>,
}

/// GET/POST /rest/getChatMessages
pub async fn get_chat_messages(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<GetChatMessagesParams>,
) -> SubsonicResponse<ChatMessagesResponse> {
    ChatMessagesResponse { chat_messages: ChatMessagesBody { chat_message: vec![] } }.into()
}

// --- addChatMessage ---

#[derive(Deserialize)]
pub struct AddChatMessageParams {
    pub message: String,
}

/// GET/POST /rest/addChatMessage
pub async fn add_chat_message(
    _auth: SubsonicAuth,
    QueryOrForm(_params): QueryOrForm<AddChatMessageParams>,
) -> SubsonicResponse<Empty> {
    Empty {}.into()
}
