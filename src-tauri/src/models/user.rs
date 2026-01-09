use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    /// Original name broadcast by the user
    pub name: String,
    /// Local alias set by the current user (overrides name in UI)
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub about: Option<String>,
    pub last_seen: Option<i64>,
    pub is_online: bool,
}
