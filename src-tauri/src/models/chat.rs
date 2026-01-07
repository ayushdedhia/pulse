use serde::{Deserialize, Serialize};

use super::message::Message;
use super::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chat {
   pub id: String,
   pub chat_type: String,
   pub name: Option<String>,
   pub avatar_url: Option<String>,
   pub created_at: i64,
   pub updated_at: i64,
   pub last_message: Option<Message>,
   pub unread_count: i32,
   pub participant: Option<User>,
}
