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
   #[serde(rename = "read_receipt")]
   ReadReceipt {
      chat_id: String,
      user_id: String,
      message_id: String,
   },
   #[serde(rename = "connect")]
   Connect { user_id: String },
   #[serde(rename = "error")]
   Error { message: String },
}
