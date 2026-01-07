use crate::db::Database;
use crate::websocket::{get_ws_server, NetworkStatus, WsMessage};
use tauri::State;

#[tauri::command]
pub fn broadcast_message(
    db: State<'_, Database>,
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

    let msg = WsMessage::ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        chat_id,
        sender_id,
        sender_name,
        content,
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
