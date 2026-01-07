use rusqlite::{Connection, Result};
use std::fs;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct Database(pub Mutex<Connection>);

pub fn init_database(app: &AppHandle) -> Result<()> {
   let app_dir = app
      .path()
      .app_data_dir()
      .expect("Failed to get app data dir");
   fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

   let db_path = app_dir.join("pulse.db");
   let conn = Connection::open(db_path)?;

   // Create tables
   conn.execute_batch(
      "
        -- Users table (contacts + self)
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
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

        -- Create indexes for better performance
        CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
        CREATE INDEX IF NOT EXISTS idx_chat_participants_user_id ON chat_participants(user_id);
        ",
   )?;

   // Create current user if not exists
   let user_count: i32 =
      conn.query_row("SELECT COUNT(*) FROM users WHERE is_self = 1", [], |row| {
         row.get(0)
      })?;

   if user_count == 0 {
      // Generate a unique user ID for this instance
      let user_id = uuid::Uuid::new_v4().to_string();
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
