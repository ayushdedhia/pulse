use crate::db::Database;
use crate::models::{Message, User};
use crate::utils::{generate_deterministic_chat_id, get_self_id};
use crate::websocket::{get_ws_server, WsMessage};
use tauri::State;

#[tauri::command]
pub fn get_messages(
    db: State<'_, Database>,
    chat_id: String,
    limit: i32,
    offset: i32,
) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                    m.reply_to_id, m.status, m.created_at, m.edited_at,
                    u.id, u.name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
             FROM messages m
             LEFT JOIN users u ON m.sender_id = u.id
             WHERE m.chat_id = ?1
             ORDER BY m.created_at ASC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let messages = stmt
        .query_map([&chat_id, &limit.to_string(), &offset.to_string()], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                sender: Some(User {
                    id: row.get(10)?,
                    name: row.get(11)?,
                    phone: row.get(12)?,
                    avatar_url: row.get(13)?,
                    about: row.get(14)?,
                    last_seen: row.get(15)?,
                    is_online: row.get::<_, i32>(16)? == 1,
                }),
                content: row.get(3)?,
                message_type: row.get(4)?,
                media_url: row.get(5)?,
                reply_to_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                edited_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

#[tauri::command]
pub fn send_message(
    db: State<'_, Database>,
    chat_id: String,
    content: String,
    message_type: String,
) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();
    let msg_id = uuid::Uuid::new_v4().to_string();

    let self_id = get_self_id(&conn)?;

    conn.execute(
        "INSERT INTO messages (id, chat_id, sender_id, content, message_type, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'sent', ?6)",
        (&msg_id, &chat_id, &self_id, &content, &message_type, now),
    )
    .map_err(|e| e.to_string())?;

    // Update chat's updated_at
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        (now, &chat_id),
    )
    .map_err(|e| e.to_string())?;

    // Get sender info
    let sender = conn
        .query_row(
            "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE is_self = 1",
            [],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    phone: row.get(2)?,
                    avatar_url: row.get(3)?,
                    about: row.get(4)?,
                    last_seen: row.get(5)?,
                    is_online: row.get::<_, i32>(6)? == 1,
                })
            },
        )
        .ok();

    Ok(Message {
        id: msg_id,
        chat_id,
        sender_id: self_id,
        sender,
        content: Some(content),
        message_type,
        media_url: None,
        reply_to_id: None,
        status: "sent".to_string(),
        created_at: now,
        edited_at: None,
    })
}

#[tauri::command]
pub fn mark_as_read(db: State<'_, Database>, chat_id: String) -> Result<Vec<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let self_id = get_self_id(&conn)?;

    // Get message IDs that will be marked as read (for read receipts)
    let mut stmt = conn
        .prepare(
            "SELECT id FROM messages WHERE chat_id = ?1 AND sender_id != ?2 AND status != 'read'",
        )
        .map_err(|e| e.to_string())?;

    let message_ids: Vec<String> = stmt
        .query_map([&chat_id, &self_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Update messages to read status
    conn.execute(
        "UPDATE messages SET status = 'read' WHERE chat_id = ?1 AND sender_id != ?2",
        [&chat_id, &self_id],
    )
    .map_err(|e| e.to_string())?;

    // Broadcast read receipt if there are messages to mark as read
    if !message_ids.is_empty() {
        let read_receipt = WsMessage::ReadReceipt {
            chat_id: chat_id.clone(),
            user_id: self_id,
            message_ids: message_ids.clone(),
        };
        let _ = get_ws_server().broadcast(read_receipt);
    }

    Ok(message_ids)
}

#[tauri::command]
pub fn update_message_status(
    db: State<'_, Database>,
    message_id: String,
    status: String,
) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE messages SET status = ?1 WHERE id = ?2",
        [&status, &message_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn search_messages(db: State<'_, Database>, query: String) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                    m.reply_to_id, m.status, m.created_at, m.edited_at,
                    u.id, u.name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
             FROM messages m
             LEFT JOIN users u ON m.sender_id = u.id
             WHERE m.content LIKE ?1
             ORDER BY m.created_at DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let messages = stmt
        .query_map([&search_pattern], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                sender: Some(User {
                    id: row.get(10)?,
                    name: row.get(11)?,
                    phone: row.get(12)?,
                    avatar_url: row.get(13)?,
                    about: row.get(14)?,
                    last_seen: row.get(15)?,
                    is_online: row.get::<_, i32>(16)? == 1,
                }),
                content: row.get(3)?,
                message_type: row.get(4)?,
                media_url: row.get(5)?,
                reply_to_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                edited_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(messages)
}

/// Receive an incoming message from WebSocket and save it to local database
#[tauri::command]
pub fn receive_message(
    db: State<'_, Database>,
    id: String,
    _chat_id: String, // Ignored - we generate deterministic chat ID from user IDs
    sender_id: String,
    sender_name: Option<String>,
    content: String,
    timestamp: i64,
) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let self_id = get_self_id(&conn)?;

    // Don't save messages from ourselves (we already have them)
    if sender_id == self_id {
        return Err("Message from self, skipping".to_string());
    }

    // Check if message already exists
    let exists: bool = conn
        .query_row("SELECT 1 FROM messages WHERE id = ?1", [&id], |_| Ok(true))
        .unwrap_or(false);

    if exists {
        return Err("Message already exists".to_string());
    }

    // Generate deterministic chat ID
    let chat_id = generate_deterministic_chat_id(&self_id, &sender_id);

    // Check if sender exists as a user, if not create them
    let sender_exists: bool = conn
        .query_row("SELECT 1 FROM users WHERE id = ?1", [&sender_id], |_| {
            Ok(true)
        })
        .unwrap_or(false);

    if !sender_exists {
        let name = sender_name
            .clone()
            .unwrap_or_else(|| format!("User {}", &sender_id[..8.min(sender_id.len())]));
        conn.execute(
            "INSERT INTO users (id, name, phone, avatar_url, about, last_seen, is_online, is_self)
             VALUES (?1, ?2, '', '', 'Hey there! I am using Pulse', ?3, 0, 0)",
            (&sender_id, &name, timestamp),
        )
        .map_err(|e| e.to_string())?;
    }

    // Check if chat exists, if not create it
    let chat_exists: bool = conn
        .query_row("SELECT 1 FROM chats WHERE id = ?1", [&chat_id], |_| {
            Ok(true)
        })
        .unwrap_or(false);

    if !chat_exists {
        conn.execute(
            "INSERT INTO chats (id, type, created_at, updated_at) VALUES (?1, 'individual', ?2, ?3)",
            (&chat_id, timestamp, timestamp),
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
            (&chat_id, &self_id, timestamp),
        )
        .map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
            (&chat_id, &sender_id, timestamp),
        )
        .map_err(|e| e.to_string())?;
    }

    // Insert the message
    conn.execute(
        "INSERT INTO messages (id, chat_id, sender_id, content, message_type, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'text', 'received', ?5)",
        (&id, &chat_id, &sender_id, &content, timestamp),
    )
    .map_err(|e| e.to_string())?;

    // Update chat's updated_at
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        (timestamp, &chat_id),
    )
    .map_err(|e| e.to_string())?;

    // Get sender info
    let sender = conn
        .query_row(
            "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE id = ?1",
            [&sender_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    phone: row.get(2)?,
                    avatar_url: row.get(3)?,
                    about: row.get(4)?,
                    last_seen: row.get(5)?,
                    is_online: row.get::<_, i32>(6)? == 1,
                })
            },
        )
        .ok();

    // Broadcast delivery receipt back to sender
    let delivery_receipt = WsMessage::DeliveryReceipt {
        message_id: id.clone(),
        chat_id: chat_id.clone(),
        delivered_to: self_id,
    };
    let _ = get_ws_server().broadcast(delivery_receipt);

    Ok(Message {
        id,
        chat_id,
        sender_id,
        sender,
        content: Some(content),
        message_type: "text".to_string(),
        media_url: None,
        reply_to_id: None,
        status: "received".to_string(),
        created_at: timestamp,
        edited_at: None,
    })
}
