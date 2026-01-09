use crate::crypto::get_crypto_manager;
use crate::db::Database;
use crate::websocket::{get_session_token, get_ws_server, NetworkStatus, WsMessage};
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

    // Encrypt the message content before broadcasting
    let encrypted_content = {
        let manager = get_crypto_manager();
        if let Some(peer_id) = get_peer_user_id(&conn, &chat_id, &sender_id) {
            if manager.ensure_session(&conn, &peer_id, &chat_id).unwrap_or(false) {
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
        content: encrypted_content,
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    get_ws_server().broadcast(msg)?;
    Ok(true)
}

#[tauri::command]
pub fn get_ws_port() -> Result<u16, String> {
    Ok(9001)
}

#[tauri::command]
pub async fn get_local_ip() -> Result<Option<String>, String> {
    Ok(get_ws_server().get_local_ip().await)
}

#[tauri::command]
pub async fn get_network_status() -> Result<NetworkStatus, String> {
    Ok(get_ws_server().get_network_status().await)
}

#[tauri::command]
pub async fn connect_to_peer(ip: String, port: Option<u16>) -> Result<(), String> {
    let port = port.unwrap_or(9001);
    get_ws_server().connect_to_peer(&ip, port).await
}

/// Get the WebSocket authentication token for this session
#[tauri::command]
pub fn get_ws_auth_token() -> Result<String, String> {
    Ok(get_session_token().to_string())
}

/// Broadcast current user's online presence to all connected peers
#[tauri::command]
pub fn broadcast_presence(user_id: String) -> Result<(), String> {
    let msg = WsMessage::Presence {
        user_id,
        is_online: true,
        last_seen: None,
    };
    get_ws_server().broadcast(msg)
}
