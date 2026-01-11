use crate::crypto::get_crypto_manager;
use crate::db::Database;
use crate::websocket::{get_ws_client, WsMessage};
use tauri::State;

/// Helper to get the peer user ID from a chat (for 1-on-1 chats)
fn get_peer_user_id(conn: &rusqlite::Connection, chat_id: &str, self_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT user_id FROM chat_participants WHERE chat_id = ?1 AND user_id != ?2 LIMIT 1",
        [chat_id, self_id],
        |row| row.get(0),
    )
    .ok()
}

#[tauri::command]
pub fn broadcast_message(
    db: State<'_, Database>,
    message_id: String,
    chat_id: String,
    content: String,
    sender_id: String,
    reply_to_id: Option<String>,
) -> Result<bool, String> {
    // Get sender's name from database
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sender_name: String = conn
        .query_row(
            "SELECT name FROM users WHERE id = ?1",
            [&sender_id],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get the recipient (peer) user ID for routing
    let recipient_id = get_peer_user_id(&conn, &chat_id, &sender_id)
        .unwrap_or_else(|| sender_id.clone()); // Fallback to sender for group chats

    // Encrypt the message content before broadcasting
    let encrypted_content = {
        let manager = get_crypto_manager();
        if let Some(peer_id) = get_peer_user_id(&conn, &chat_id, &sender_id) {
            if manager
                .ensure_session(&conn, &peer_id, &chat_id)
                .unwrap_or(false)
            {
                if let Ok(encrypted) = manager.encrypt(&content, &chat_id) {
                    if let Ok(json) = serde_json::to_string(&encrypted) {
                        format!("enc:{}", json)
                    } else {
                        content.clone()
                    }
                } else {
                    content.clone()
                }
            } else {
                content.clone()
            }
        } else {
            content.clone()
        }
    };

    let msg = WsMessage::ChatMessage {
        id: message_id,
        chat_id,
        sender_id,
        sender_name,
        recipient_id,
        content: encrypted_content,
        timestamp: chrono::Utc::now().timestamp_millis(),
        reply_to_id,
    };

    get_ws_client().broadcast(msg)?;
    Ok(true)
}

/// Get the central server URL
#[tauri::command]
pub async fn get_server_url() -> Result<String, String> {
    Ok(get_ws_client().get_server_url().await)
}

/// Check if connected to the central server
#[tauri::command]
pub async fn is_connected() -> Result<bool, String> {
    Ok(get_ws_client().is_connected().await)
}

/// Connect to the central WebSocket server
#[tauri::command]
pub async fn connect_websocket(user_id: String) -> Result<(), String> {
    crate::websocket::init_websocket(&user_id).await
}

/// Gracefully disconnect from the central WebSocket server
#[tauri::command]
pub fn disconnect_websocket() {
    get_ws_client().disconnect()
}

/// Broadcast current user's online presence to all connected peers
#[tauri::command]
pub fn broadcast_presence(user_id: String) -> Result<(), String> {
    let msg = WsMessage::Presence {
        user_id,
        is_online: true,
        last_seen: None,
    };
    get_ws_client().broadcast(msg)
}

/// Broadcast profile update to all connected peers
#[tauri::command]
pub fn broadcast_profile(
    user_id: String,
    name: String,
    phone: Option<String>,
    avatar_url: Option<String>,
    about: Option<String>,
    avatar_data: Option<String>,
) -> Result<(), String> {
    let msg = WsMessage::ProfileUpdate {
        user_id,
        name,
        phone,
        avatar_url,
        about,
        avatar_data,
    };
    get_ws_client().broadcast(msg)
}
