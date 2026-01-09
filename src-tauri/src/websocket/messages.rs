use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "message")]
    ChatMessage {
        id: String,
        chat_id: String,
        sender_id: String,
        sender_name: String,
        content: String,
        timestamp: i64,
    },
    #[serde(rename = "typing")]
    Typing {
        chat_id: String,
        user_id: String,
        is_typing: bool,
    },
    #[serde(rename = "presence")]
    Presence {
        user_id: String,
        is_online: bool,
        last_seen: Option<i64>,
    },
    #[serde(rename = "delivery_receipt")]
    DeliveryReceipt {
        message_id: String,
        chat_id: String,
        delivered_to: String,
    },
    #[serde(rename = "read_receipt")]
    ReadReceipt {
        chat_id: String,
        user_id: String,
        message_ids: Vec<String>,
    },
    #[serde(rename = "connect")]
    Connect {
        user_id: String,
        /// Authentication token - required for WebSocket connection
        #[serde(default)]
        auth_token: Option<String>,
    },
    #[serde(rename = "auth_response")]
    AuthResponse {
        success: bool,
        message: String,
    },
    #[serde(rename = "error")]
    Error { message: String },
    /// Peer-to-peer connection with token exchange
    #[serde(rename = "peer_connect")]
    PeerConnect {
        /// The connecting peer's token to be added to trusted tokens
        peer_token: String,
    },
    /// Profile update broadcast to peers
    #[serde(rename = "profile_update")]
    ProfileUpdate {
        user_id: String,
        name: String,
        phone: Option<String>,
        avatar_url: Option<String>,
        about: Option<String>,
        /// Base64-encoded avatar image bytes (only when avatar changes)
        avatar_data: Option<String>,
    },
}
