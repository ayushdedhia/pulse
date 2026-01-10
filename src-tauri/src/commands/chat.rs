use crate::db::Database;
use crate::models::input::{CreateChatInput, ValidateExt};
use crate::models::{Chat, Message, User};
use crate::utils::{generate_deterministic_chat_id, get_self_id};
use tauri::State;

#[tauri::command]
pub fn get_chats(db: State<'_, Database>) -> Result<Vec<Chat>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let self_id = get_self_id(&conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.type, c.name, c.avatar_url, c.created_at, c.updated_at
         FROM chats c
         ORDER BY c.updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let chats: Vec<Chat> = stmt
        .query_map([], |row| {
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
        })
        .map_err(|e| e.to_string())?
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
         ORDER BY m.created_at DESC LIMIT 1",
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
            |row| row.get::<_, i32>(0),
        ) {
            chat.unread_count = count;
        }

        // For individual chats, get the other participant
        if chat.chat_type == "individual" {
            if let Ok(mut user_stmt) = conn.prepare(
                "SELECT u.id, u.name, u.display_name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
            FROM users u
            JOIN chat_participants cp ON u.id = cp.user_id
            WHERE cp.chat_id = ?1 AND u.id != ?2",
            ) {
                if let Ok(user) = user_stmt.query_row([&chat.id, &self_id], |row| {
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
pub fn create_chat(db: State<'_, Database>, input: CreateChatInput) -> Result<Chat, String> {
    input.validate_input()?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();

    let self_id = get_self_id(&conn)?;
    let chat_id = generate_deterministic_chat_id(&self_id, &input.user_id);

    // Check if chat already exists
    let existing: Option<String> = conn
        .query_row("SELECT id FROM chats WHERE id = ?1", [&chat_id], |row| {
            row.get(0)
        })
        .ok();

    if existing.is_some() {
        return get_chat_by_id(&conn, &chat_id, &self_id);
    }

    conn.execute(
        "INSERT INTO chats (id, type, created_at, updated_at) VALUES (?1, 'individual', ?2, ?3)",
        (&chat_id, now, now),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
        (&chat_id, &self_id, now),
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO chat_participants (chat_id, user_id, joined_at) VALUES (?1, ?2, ?3)",
        (&chat_id, &input.user_id, now),
    )
    .map_err(|e| e.to_string())?;

    get_chat_by_id(&conn, &chat_id, &self_id)
}

/// Helper function to get a chat by ID with participant info
pub fn get_chat_by_id(
    conn: &rusqlite::Connection,
    chat_id: &str,
    self_id: &str,
) -> Result<Chat, String> {
    let mut chat: Chat = conn
        .query_row(
            "SELECT id, type, name, avatar_url, created_at, updated_at FROM chats WHERE id = ?1",
            [chat_id],
            |row| {
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
            },
        )
        .map_err(|e| e.to_string())?;

    // Get participant for individual chats
    if chat.chat_type == "individual" {
        if let Ok(user) = conn.query_row(
            "SELECT u.id, u.name, u.display_name, u.phone, u.avatar_url, u.about, u.last_seen, u.is_online
         FROM users u
         JOIN chat_participants cp ON u.id = cp.user_id
         WHERE cp.chat_id = ?1 AND u.id != ?2",
            [chat_id, self_id],
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
        ) {
            chat.participant = Some(user);
        }
    }

    Ok(chat)
}
