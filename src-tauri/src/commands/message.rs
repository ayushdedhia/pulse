use crate::crypto::get_crypto_manager;
use crate::db::Database;
use crate::models::input::{
    GetMessagesInput, MarkAsReadInput, SearchMessagesInput, SendMessageInput,
    UpdateMessageStatusInput, ValidateExt,
};
use crate::models::{Message, User};
use crate::utils::validation::validate_user_id;
use crate::utils::{generate_deterministic_chat_id, get_self_id};
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

/// Encrypt message content if a session exists for the chat
fn encrypt_content(
    conn: &rusqlite::Connection,
    content: &str,
    chat_id: &str,
    self_id: &str,
) -> Result<String, String> {
    let manager = get_crypto_manager();

    // Try to get peer's user ID and ensure session exists
    if let Some(peer_id) = get_peer_user_id(conn, chat_id, self_id) {
        if manager.ensure_session(conn, &peer_id, chat_id)? {
            // Session exists, encrypt the message
            let encrypted = manager.encrypt(content, chat_id)?;
            let json = serde_json::to_string(&encrypted).map_err(|e| e.to_string())?;
            return Ok(format!("enc:{}", json));
        }
    }

    // No session available, store as plaintext (for backward compatibility)
    // In production, you might want to reject unencrypted messages
    Ok(content.to_string())
}

/// Decrypt message content if it's encrypted
fn decrypt_content(
    conn: &rusqlite::Connection,
    content: &str,
    chat_id: &str,
    self_id: &str,
) -> String {
    // Check if content is encrypted (prefixed with "enc:")
    if let Some(encrypted_json) = content.strip_prefix("enc:") {
        let manager = get_crypto_manager();

        // Try to ensure session exists
        if let Some(peer_id) = get_peer_user_id(conn, chat_id, self_id) {
            let _ = manager.ensure_session(conn, &peer_id, chat_id);
        }

        // Try to decrypt
        if let Ok(encrypted) = serde_json::from_str(encrypted_json) {
            if let Ok(plaintext) = manager.decrypt(&encrypted, chat_id) {
                return plaintext;
            }
        }

        // Decryption failed - return placeholder
        "[Encrypted message - unable to decrypt]".to_string()
    } else {
        // Not encrypted, return as-is
        content.to_string()
    }
}

#[tauri::command]
pub fn get_messages(db: State<'_, Database>, input: GetMessagesInput) -> Result<Vec<Message>, String> {
    input.validate_input()?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let self_id = get_self_id(&conn)?;
    let chat_id = &input.chat_id;

    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                    m.reply_to_id, m.status, m.created_at, m.edited_at,
                    u.id, u.name, u.display_name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
             FROM messages m
             LEFT JOIN users u ON m.sender_id = u.id
             WHERE m.chat_id = ?1
             ORDER BY m.created_at ASC
             LIMIT ?2 OFFSET ?3",
        )
        .map_err(|e| e.to_string())?;

    let messages: Vec<Message> = stmt
        .query_map([chat_id, &input.limit.to_string(), &input.offset.to_string()], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                sender: Some(User {
                    id: row.get(10)?,
                    name: row.get(11)?,
                    display_name: row.get(12)?,
                    phone: row.get(13)?,
                    avatar_url: row.get(14)?,
                    about: row.get(15)?,
                    last_seen: row.get(16)?,
                    is_online: row.get::<_, i32>(17)? == 1,
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

    // Decrypt messages
    let decrypted_messages: Vec<Message> = messages
        .into_iter()
        .map(|mut msg| {
            if let Some(ref content) = msg.content {
                msg.content = Some(decrypt_content(&conn, content, chat_id, &self_id));
            }
            msg
        })
        .collect();

    Ok(decrypted_messages)
}

#[tauri::command]
pub fn send_message(db: State<'_, Database>, input: SendMessageInput) -> Result<Message, String> {
    input.validate_input()?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let chat_id = &input.chat_id;
    let now = chrono::Utc::now().timestamp_millis();
    let msg_id = uuid::Uuid::new_v4().to_string();

    let self_id = get_self_id(&conn)?;

    // Encrypt the message content before storing
    let encrypted_content = encrypt_content(&conn, &input.content, chat_id, &self_id)?;

    conn.execute(
        "INSERT INTO messages (id, chat_id, sender_id, content, message_type, reply_to_id, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'sent', ?7)",
        (&msg_id, chat_id, &self_id, &encrypted_content, &input.message_type, &input.reply_to_id, now),
    )
    .map_err(|e| e.to_string())?;

    // Update chat's updated_at
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        (now, chat_id),
    )
    .map_err(|e| e.to_string())?;

    // Get sender info
    let sender = conn
        .query_row(
            "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online FROM users WHERE is_self = 1",
            [],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    phone: row.get(3)?,
                    avatar_url: row.get(4)?,
                    about: row.get(5)?,
                    last_seen: row.get(6)?,
                    is_online: row.get::<_, i32>(7)? == 1,
                })
            },
        )
        .ok();

    Ok(Message {
        id: msg_id,
        chat_id: input.chat_id,
        sender_id: self_id,
        sender,
        content: Some(input.content),
        message_type: input.message_type,
        media_url: None,
        reply_to_id: input.reply_to_id,
        status: "sent".to_string(),
        created_at: now,
        edited_at: None,
    })
}

#[tauri::command]
pub fn mark_as_read(db: State<'_, Database>, input: MarkAsReadInput) -> Result<Vec<String>, String> {
    input.validate_input()?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let chat_id = &input.chat_id;

    let self_id = get_self_id(&conn)?;

    // Get message IDs that will be marked as read (for read receipts)
    let mut stmt = conn
        .prepare(
            "SELECT id FROM messages WHERE chat_id = ?1 AND sender_id != ?2 AND status != 'read'",
        )
        .map_err(|e| e.to_string())?;

    let message_ids: Vec<String> = stmt
        .query_map([chat_id, &self_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Update messages to read status
    conn.execute(
        "UPDATE messages SET status = 'read' WHERE chat_id = ?1 AND sender_id != ?2",
        [chat_id, &self_id],
    )
    .map_err(|e| e.to_string())?;

    // Broadcast read receipt if there are messages to mark as read
    if !message_ids.is_empty() {
        let read_receipt = WsMessage::ReadReceipt {
            chat_id: input.chat_id,
            user_id: self_id,
            message_ids: message_ids.clone(),
        };
        let _ = get_ws_client().broadcast(read_receipt);
    }

    Ok(message_ids)
}

#[tauri::command]
pub fn update_message_status(
    db: State<'_, Database>,
    input: UpdateMessageStatusInput,
) -> Result<bool, String> {
    input.validate_input()?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE messages SET status = ?1 WHERE id = ?2",
        [&input.status, &input.message_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Search messages by content
/// Note: With E2E encryption, searching within encrypted content has limitations.
/// Only unencrypted messages or messages that match the encrypted pattern will be found.
/// For full search capability, consider implementing a local search index.
#[tauri::command]
pub fn search_messages(db: State<'_, Database>, input: SearchMessagesInput) -> Result<Vec<Message>, String> {
    input.validate_input()?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let self_id = get_self_id(&conn)?;
    let search_pattern = format!("%{}%", input.query);

    let mut stmt = conn
        .prepare(
            "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                    m.reply_to_id, m.status, m.created_at, m.edited_at,
                    u.id, u.name, u.display_name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
             FROM messages m
             LEFT JOIN users u ON m.sender_id = u.id
             WHERE m.content LIKE ?1
             ORDER BY m.created_at DESC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let messages: Vec<Message> = stmt
        .query_map([&search_pattern], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                sender: Some(User {
                    id: row.get(10)?,
                    name: row.get(11)?,
                    display_name: row.get(12)?,
                    phone: row.get(13)?,
                    avatar_url: row.get(14)?,
                    about: row.get(15)?,
                    last_seen: row.get(16)?,
                    is_online: row.get::<_, i32>(17)? == 1,
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

    // Decrypt messages for display
    let decrypted_messages: Vec<Message> = messages
        .into_iter()
        .map(|mut msg| {
            if let Some(ref content) = msg.content {
                msg.content = Some(decrypt_content(&conn, content, &msg.chat_id, &self_id));
            }
            msg
        })
        .collect();

    Ok(decrypted_messages)
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
    reply_to_id: Option<String>,
) -> Result<Message, String> {
    // Validate input
    validate_user_id(&sender_id)?;
    // Note: content might be encrypted so we skip content validation here
    // The encryption layer handles its own size limits

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

    // The content might be encrypted (prefixed with "enc:") from the sender
    // Store as-is in the database (preserving encryption)
    conn.execute(
        "INSERT INTO messages (id, chat_id, sender_id, content, message_type, reply_to_id, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'text', ?5, 'received', ?6)",
        (&id, &chat_id, &sender_id, &content, &reply_to_id, timestamp),
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
            "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online FROM users WHERE id = ?1",
            [&sender_id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    phone: row.get(3)?,
                    avatar_url: row.get(4)?,
                    about: row.get(5)?,
                    last_seen: row.get(6)?,
                    is_online: row.get::<_, i32>(7)? == 1,
                })
            },
        )
        .ok();

    // Broadcast delivery receipt back to sender
    let delivery_receipt = WsMessage::DeliveryReceipt {
        message_id: id.clone(),
        chat_id: chat_id.clone(),
        delivered_to: self_id.clone(),
    };
    let _ = get_ws_client().broadcast(delivery_receipt);

    // Decrypt content for the returned message (so UI can display it)
    let decrypted_content = decrypt_content(&conn, &content, &chat_id, &self_id);

    Ok(Message {
        id,
        chat_id,
        sender_id,
        sender,
        content: Some(decrypted_content),
        message_type: "text".to_string(),
        media_url: None,
        reply_to_id,
        status: "received".to_string(),
        created_at: timestamp,
        edited_at: None,
    })
}
