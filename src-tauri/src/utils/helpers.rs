use rusqlite::Connection;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Get the current user's ID from the database
pub fn get_self_id(conn: &Connection) -> Result<String, String> {
    conn.query_row("SELECT id FROM users WHERE is_self = 1", [], |row| {
        row.get(0)
    })
    .map_err(|e| e.to_string())
}

/// Generate a deterministic chat ID from two user IDs
/// This ensures both users will have the same chat ID regardless of who initiates
pub fn generate_deterministic_chat_id(user_id_1: &str, user_id_2: &str) -> String {
    let mut ids = vec![user_id_1.to_string(), user_id_2.to_string()];
    ids.sort();
    let chat_id = format!("chat_{}_{}", ids[0], ids[1]);

    let mut hasher = DefaultHasher::new();
    chat_id.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
