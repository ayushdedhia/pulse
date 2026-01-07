use serde::{Deserialize, Serialize};

use super::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
   pub id: String,
   pub chat_id: String,
   pub sender_id: String,
   pub sender: Option<User>,
   pub content: Option<String>,
   pub message_type: String,
   pub media_url: Option<String>,
   pub reply_to_id: Option<String>,
   pub status: String,
   pub created_at: i64,
   pub edited_at: Option<i64>,
}
