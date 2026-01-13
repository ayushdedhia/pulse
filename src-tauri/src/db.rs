use rusqlite::{Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct Database(pub Mutex<Connection>);

#[derive(Debug, Serialize, Deserialize)]
struct StoredIdentity {
    user_id: String,
}

fn load_stored_identity(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }

    let contents = fs::read_to_string(path).ok()?;
    let identity: StoredIdentity = serde_json::from_str(&contents).ok()?;
    let trimmed = identity.user_id.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn save_stored_identity(path: &Path, user_id: &str) {
    let identity = StoredIdentity {
        user_id: user_id.to_string(),
    };
    let contents =
        serde_json::to_string_pretty(&identity).expect("Failed to serialize identity");
    fs::write(path, contents).expect("Failed to write identity file");
}

pub fn init_database(app: &AppHandle) -> Result<()> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

    let db_path = app_dir.join("pulse.db");
    let identity_path = app_dir.join("identity.json");
    let conn = Connection::open(db_path)?;

    // Create tables
    conn.execute_batch(
        "
        -- Users table (contacts + self)
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            display_name TEXT,
            phone TEXT,
            avatar_url TEXT,
            about TEXT DEFAULT 'Hey there! I am using Pulse',
            last_seen INTEGER,
            is_online INTEGER DEFAULT 0,
            is_self INTEGER DEFAULT 0
        );

        -- Chats table (1-on-1 and groups)
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            type TEXT CHECK(type IN ('individual', 'group')) NOT NULL,
            name TEXT,
            avatar_url TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- Chat participants
        CREATE TABLE IF NOT EXISTS chat_participants (
            chat_id TEXT REFERENCES chats(id),
            user_id TEXT REFERENCES users(id),
            role TEXT DEFAULT 'member',
            joined_at INTEGER,
            PRIMARY KEY (chat_id, user_id)
        );

        -- Messages table
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT REFERENCES chats(id),
            sender_id TEXT REFERENCES users(id),
            content TEXT,
            message_type TEXT DEFAULT 'text',
            media_url TEXT,
            reply_to_id TEXT REFERENCES messages(id),
            status TEXT DEFAULT 'sent',
            created_at INTEGER NOT NULL,
            edited_at INTEGER
        );

        -- Public keys table (identity + peers) for E2E encryption
        CREATE TABLE IF NOT EXISTS public_keys (
            user_id TEXT PRIMARY KEY REFERENCES users(id),
            public_key BLOB NOT NULL,
            key_type TEXT CHECK(key_type IN ('identity', 'peer')) NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        -- URL previews cache table
        CREATE TABLE IF NOT EXISTS url_previews (
            url TEXT PRIMARY KEY,
            title TEXT,
            description TEXT,
            image_url TEXT,
            site_name TEXT,
            fetched_at INTEGER NOT NULL
        );

        -- Create indexes for better performance
        CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
        CREATE INDEX IF NOT EXISTS idx_chat_participants_user_id ON chat_participants(user_id);
        CREATE INDEX IF NOT EXISTS idx_public_keys_type ON public_keys(key_type);
        ",
    )?;

    // Migration: Add display_name column if it doesn't exist (for existing databases)
    let has_display_name: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'display_name'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !has_display_name {
        conn.execute("ALTER TABLE users ADD COLUMN display_name TEXT", [])?;
    }

    // Migration: Add link_previews_enabled column to users table
    let has_link_previews: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'link_previews_enabled'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !has_link_previews {
        conn.execute(
            "ALTER TABLE users ADD COLUMN link_previews_enabled INTEGER DEFAULT 1",
            [],
        )?;
    }

    // Migration: Add preview_url column to messages table
    let has_preview_url: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('messages') WHERE name = 'preview_url'",
            [],
            |row| row.get::<_, i32>(0),
        )
        .map(|count| count > 0)
        .unwrap_or(false);

    if !has_preview_url {
        conn.execute("ALTER TABLE messages ADD COLUMN preview_url TEXT", [])?;
    }

    // Create or reuse current user with a stable identity ID
    let existing_self_id: Option<String> = conn
        .query_row(
            "SELECT id FROM users WHERE is_self = 1 LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(self_id) = existing_self_id {
        if load_stored_identity(&identity_path).as_deref() != Some(self_id.as_str()) {
            save_stored_identity(&identity_path, &self_id);
        }
    } else {
        let user_id = load_stored_identity(&identity_path).unwrap_or_else(|| {
            let new_id = uuid::Uuid::new_v4().to_string();
            save_stored_identity(&identity_path, &new_id);
            new_id
        });
        // Create a default name using part of the UUID for uniqueness
        let default_name = format!("User {}", &user_id[..8]);
        conn.execute(
            "INSERT INTO users (id, name, phone, avatar_url, about, is_self, is_online) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &user_id,
                &default_name,
                "",
                "",
                "Hey there! I am using Pulse",
                1,
                1,
            ),
        )?;
    }

    app.manage(Database(Mutex::new(conn)));
    Ok(())
}
