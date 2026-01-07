use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub about: Option<String>,
    pub last_seen: Option<i64>,
    pub is_online: bool,
}
