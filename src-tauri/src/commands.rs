use crate::db::Database;
use serde::{Deserialize, Serialize};
use tauri::State;

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

#[tauri::command]
pub fn get_chats(db: State<'_, Database>) -> Result<Vec<Chat>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get current user's ID
    let self_id: String = conn.query_row(
        "SELECT id FROM users WHERE is_self = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.type, c.name, c.avatar_url, c.created_at, c.updated_at
         FROM chats c
         ORDER BY c.updated_at DESC"
    ).map_err(|e| e.to_string())?;

    let chats: Vec<Chat> = stmt.query_map([], |row| {
        Ok(Chat {
            id: row.get(0)?,
            chat_type: row.get(1)?,
            name: row.get(2)?,
            avatar_url: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            last_message: None,
            unread_count: 0,
            participant: None,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    // Enrich chats with additional data
    let mut enriched_chats = Vec::new();
    for mut chat in chats {
        // Get last message
        if let Ok(mut msg_stmt) = conn.prepare(
            "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                    m.reply_to_id, m.status, m.created_at, m.edited_at
             FROM messages m
             WHERE m.chat_id = ?1
             ORDER BY m.created_at DESC LIMIT 1"
        ) {
            if let Ok(msg) = msg_stmt.query_row([&chat.id], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    chat_id: row.get(1)?,
                    sender_id: row.get(2)?,
                    sender: None,
                    content: row.get(3)?,
                    message_type: row.get(4)?,
                    media_url: row.get(5)?,
                    reply_to_id: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    edited_at: row.get(9)?,
                })
            }) {
                chat.last_message = Some(msg);
            }
        }

        // Get unread count
        if let Ok(count) = conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE chat_id = ?1 AND sender_id != ?2 AND status != 'read'",
            [&chat.id, &self_id],
            |row| row.get::<_, i32>(0)
        ) {
            chat.unread_count = count;
        }

        // For individual chats, get the other participant
        if chat.chat_type == "individual" {
            if let Ok(mut user_stmt) = conn.prepare(
                "SELECT u.id, u.name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
                 FROM users u
                 JOIN chat_participants cp ON u.id = cp.user_id
                 WHERE cp.chat_id = ?1 AND u.id != ?2"
            ) {
                if let Ok(user) = user_stmt.query_row([&chat.id, &self_id], |row| {
                    Ok(User {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        phone: row.get(2)?,
                        avatar_url: row.get(3)?,
                        about: row.get(4)?,
                        last_seen: row.get(5)?,
                        is_online: row.get::<_, i32>(6)? == 1,
                    })
                }) {
                    chat.participant = Some(user);
                }
            }
        }

        enriched_chats.push(chat);
    }

    Ok(enriched_chats)
}

#[tauri::command]
pub fn get_messages(db: State<'_, Database>, chat_id: String, limit: i32, offset: i32) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                m.reply_to_id, m.status, m.created_at, m.edited_at,
                u.id, u.name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
         FROM messages m
         LEFT JOIN users u ON m.sender_id = u.id
         WHERE m.chat_id = ?1
         ORDER BY m.created_at ASC
         LIMIT ?2 OFFSET ?3"
    ).map_err(|e| e.to_string())?;

    let messages = stmt.query_map([&chat_id, &limit.to_string(), &offset.to_string()], |row| {
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
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(messages)
}

#[tauri::command]
pub fn send_message(db: State<'_, Database>, chat_id: String, content: String, message_type: String) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();
    let msg_id = uuid::Uuid::new_v4().to_string();

    // Get current user's ID
    let self_id: String = conn.query_row(
        "SELECT id FROM users WHERE is_self = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO messages (id, chat_id, sender_id, content, message_type, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'sent', ?6)",
        (&msg_id, &chat_id, &self_id, &content, &message_type, now),
    ).map_err(|e| e.to_string())?;

    // Update chat's updated_at
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        (now, &chat_id),
    ).map_err(|e| e.to_string())?;

    // Get sender info
    let sender = conn.query_row(
        "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE is_self = 1",
        [],
        |row| Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            avatar_url: row.get(3)?,
            about: row.get(4)?,
            last_seen: row.get(5)?,
            is_online: row.get::<_, i32>(6)? == 1,
        })
    ).ok();

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
pub fn mark_as_read(db: State<'_, Database>, chat_id: String) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get current user's ID
    let self_id: String = conn.query_row(
        "SELECT id FROM users WHERE is_self = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE messages SET status = 'read' WHERE chat_id = ?1 AND sender_id != ?2",
        [&chat_id, &self_id],
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn create_chat(db: State<'_, Database>, user_id: String) -> Result<Chat, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();

    // Get current user's ID
    let self_id: String = conn.query_row(
        "SELECT id FROM users WHERE is_self = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    // Generate deterministic chat ID from both user IDs (sorted to be consistent)
    // This ensures both users will have the same chat ID
    let mut ids = vec![self_id.clone(), user_id.clone()];
    ids.sort();
    let chat_id = format!("chat_{}_{}", ids[0], ids[1]);

    // Use a shorter hash for cleaner IDs
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    chat_id.hash(&mut hasher);
    let chat_id = format!("{:016x}", hasher.finish());

    // Check if chat already exists
    let existing: Option<String> = conn.query_row(
        "SELECT id FROM chats WHERE id = ?1",
        [&chat_id],
        |row| row.get(0)
    ).ok();

    if existing.is_some() {
        // Return existing chat
        return get_chat_by_id(&conn, &chat_id, &self_id);
    }

    conn.execute(
        "INSERT INTO chats (id, type, created_at, updated_at) VALUES (?1, 'individual', ?2, ?3)",
        (&chat_id, now, now),
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
        (&chat_id, &self_id, now),
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
        (&chat_id, &user_id, now),
    ).map_err(|e| e.to_string())?;

    get_chat_by_id(&conn, &chat_id, &self_id)
}

fn get_chat_by_id(conn: &rusqlite::Connection, chat_id: &str, self_id: &str) -> Result<Chat, String> {
    let mut chat: Chat = conn.query_row(
        "SELECT id, type, name, avatar_url, created_at, updated_at FROM chats WHERE id = ?1",
        [chat_id],
        |row| Ok(Chat {
            id: row.get(0)?,
            chat_type: row.get(1)?,
            name: row.get(2)?,
            avatar_url: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            last_message: None,
            unread_count: 0,
            participant: None,
        })
    ).map_err(|e| e.to_string())?;

    // Get participant for individual chats
    if chat.chat_type == "individual" {
        if let Ok(user) = conn.query_row(
            "SELECT u.id, u.name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
             FROM users u
             JOIN chat_participants cp ON u.id = cp.user_id
             WHERE cp.chat_id = ?1 AND u.id != ?2",
            [chat_id, self_id],
            |row| Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                avatar_url: row.get(3)?,
                about: row.get(4)?,
                last_seen: row.get(5)?,
                is_online: row.get::<_, i32>(6)? == 1,
            })
        ) {
            chat.participant = Some(user);
        }
    }

    Ok(chat)
}

#[tauri::command]
pub fn get_user(db: State<'_, Database>, user_id: String) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE id = ?1",
        [&user_id],
        |row| Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            avatar_url: row.get(3)?,
            about: row.get(4)?,
            last_seen: row.get(5)?,
            is_online: row.get::<_, i32>(6)? == 1,
        })
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_user(db: State<'_, Database>) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE is_self = 1",
        [],
        |row| Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            avatar_url: row.get(3)?,
            about: row.get(4)?,
            last_seen: row.get(5)?,
            is_online: row.get::<_, i32>(6)? == 1,
        })
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_user(db: State<'_, Database>, user: User) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE users SET name = ?1, avatar_url = ?2, about = ?3 WHERE id = ?4",
        (&user.name, &user.avatar_url, &user.about, &user.id),
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn search_messages(db: State<'_, Database>, query: String) -> Result<Vec<Message>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT m.id, m.chat_id, m.sender_id, m.content, m.message_type, m.media_url,
                m.reply_to_id, m.status, m.created_at, m.edited_at,
                u.id, u.name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
         FROM messages m
         LEFT JOIN users u ON m.sender_id = u.id
         WHERE m.content LIKE ?1
         ORDER BY m.created_at DESC
         LIMIT 50"
    ).map_err(|e| e.to_string())?;

    let messages = stmt.query_map([&search_pattern], |row| {
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
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(messages)
}

#[tauri::command]
pub fn get_contacts(db: State<'_, Database>) -> Result<Vec<User>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, name, phone, avatar_url, about, last_seen, is_online
         FROM users
         WHERE is_self = 0
         ORDER BY name"
    ).map_err(|e| e.to_string())?;

    let users = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            avatar_url: row.get(3)?,
            about: row.get(4)?,
            last_seen: row.get(5)?,
            is_online: row.get::<_, i32>(6)? == 1,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(users)
}

// WebSocket commands
#[tauri::command]
pub fn broadcast_message(db: State<'_, Database>, chat_id: String, content: String, sender_id: String) -> Result<bool, String> {
    use crate::websocket::{get_ws_server, WsMessage};

    // Get sender's name from database
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let sender_name: String = conn.query_row(
        "SELECT name FROM users WHERE id = ?1",
        [&sender_id],
        |row| row.get(0)
    ).unwrap_or_else(|_| "Unknown".to_string());

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
pub fn add_contact(db: State<'_, Database>, id: String, name: String, phone: Option<String>) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();

    // Check if contact already exists
    let exists: bool = conn.query_row(
        "SELECT 1 FROM users WHERE id = ?1",
        [&id],
        |_| Ok(true)
    ).unwrap_or(false);

    if exists {
        return Err("Contact already exists".to_string());
    }

    conn.execute(
        "INSERT INTO users (id, name, phone, avatar_url, about, last_seen, is_online, is_self)
         VALUES (?1, ?2, ?3, '', 'Hey there! I am using Pulse', ?4, 0, 0)",
        (&id, &name, &phone, now),
    ).map_err(|e| e.to_string())?;

    Ok(User {
        id,
        name,
        phone,
        avatar_url: Some("".to_string()),
        about: Some("Hey there! I am using Pulse".to_string()),
        last_seen: Some(now),
        is_online: false,
    })
}

/// Receive an incoming message from WebSocket and save it to local database
#[tauri::command]
pub fn receive_message(
    db: State<'_, Database>,
    id: String,
    _chat_id: String,  // Ignored - we generate deterministic chat ID from user IDs
    sender_id: String,
    sender_name: Option<String>,
    content: String,
    timestamp: i64,
) -> Result<Message, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get current user's ID to avoid saving our own messages
    let self_id: String = conn.query_row(
        "SELECT id FROM users WHERE is_self = 1",
        [],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    // Don't save messages from ourselves (we already have them)
    if sender_id == self_id {
        return Err("Message from self, skipping".to_string());
    }

    // Check if message already exists
    let exists: bool = conn.query_row(
        "SELECT 1 FROM messages WHERE id = ?1",
        [&id],
        |_| Ok(true)
    ).unwrap_or(false);

    if exists {
        return Err("Message already exists".to_string());
    }

    // Generate deterministic chat ID from both user IDs (sorted to be consistent)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut ids = vec![self_id.clone(), sender_id.clone()];
    ids.sort();
    let expected_chat_id = format!("chat_{}_{}", ids[0], ids[1]);
    let mut hasher = DefaultHasher::new();
    expected_chat_id.hash(&mut hasher);
    let expected_chat_id = format!("{:016x}", hasher.finish());

    // Use the expected chat ID (the one both parties should have)
    let chat_id = expected_chat_id;

    // Check if sender exists as a user, if not create them
    let sender_exists: bool = conn.query_row(
        "SELECT 1 FROM users WHERE id = ?1",
        [&sender_id],
        |_| Ok(true)
    ).unwrap_or(false);

    if !sender_exists {
        // Create the sender as a contact - use sender_name if provided, otherwise generate from ID
        let name = sender_name.clone().unwrap_or_else(|| format!("User {}", &sender_id[..8.min(sender_id.len())]));
        conn.execute(
            "INSERT INTO users (id, name, phone, avatar_url, about, last_seen, is_online, is_self)
             VALUES (?1, ?2, '', '', 'Hey there! I am using Pulse', ?3, 0, 0)",
            (&sender_id, &name, timestamp),
        ).map_err(|e| e.to_string())?;
    }

    // Check if chat exists, if not create it
    let chat_exists: bool = conn.query_row(
        "SELECT 1 FROM chats WHERE id = ?1",
        [&chat_id],
        |_| Ok(true)
    ).unwrap_or(false);

    if !chat_exists {
        // Create the chat
        conn.execute(
            "INSERT INTO chats (id, type, created_at, updated_at) VALUES (?1, 'individual', ?2, ?3)",
            (&chat_id, timestamp, timestamp),
        ).map_err(|e| e.to_string())?;

        // Add participants
        conn.execute(
            "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
            (&chat_id, &self_id, timestamp),
        ).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
            (&chat_id, &sender_id, timestamp),
        ).map_err(|e| e.to_string())?;
    }

    // Insert the message
    conn.execute(
        "INSERT INTO messages (id, chat_id, sender_id, content, message_type, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'text', 'received', ?5)",
        (&id, &chat_id, &sender_id, &content, timestamp),
    ).map_err(|e| e.to_string())?;

    // Update chat's updated_at
    conn.execute(
        "UPDATE chats SET updated_at = ?1 WHERE id = ?2",
        (timestamp, &chat_id),
    ).map_err(|e| e.to_string())?;

    // Get sender info
    let sender = conn.query_row(
        "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE id = ?1",
        [&sender_id],
        |row| Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            avatar_url: row.get(3)?,
            about: row.get(4)?,
            last_seen: row.get(5)?,
            is_online: row.get::<_, i32>(6)? == 1,
        })
    ).ok();

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
