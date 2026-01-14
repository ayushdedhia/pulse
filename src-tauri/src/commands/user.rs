use crate::db::Database;
use crate::crypto::storage;
use crate::models::User;
use crate::utils::validation::{
    validate_about, validate_phone, validate_phone_id, validate_url, validate_user_name,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
struct StoredIdentity {
    user_id: String,
}

fn write_identity_file(app: &AppHandle, user_id: &str) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let identity_path = app_dir.join("identity.json");
    let identity = StoredIdentity {
        user_id: user_id.to_string(),
    };
    let contents = serde_json::to_string_pretty(&identity).map_err(|e| e.to_string())?;
    fs::write(identity_path, contents).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_user(db: State<'_, Database>, user_id: String) -> Result<User, String> {
    // Validate input (user ID can be phone number with + prefix)
    validate_phone_id(&user_id)?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online, link_previews_enabled FROM users WHERE id = ?1",
        [&user_id],
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
                link_previews_enabled: row.get::<_, i32>(8).unwrap_or(1) == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_user(db: State<'_, Database>) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online, link_previews_enabled FROM users WHERE is_self = 1",
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
                link_previews_enabled: row.get::<_, i32>(8).unwrap_or(1) == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_user(db: State<'_, Database>, user: User) -> Result<bool, String> {
    // Validate input (user ID can be phone number with + prefix)
    validate_phone_id(&user.id)?;
    validate_user_name(&user.name)?;
    validate_about(user.about.as_deref())?;
    validate_url(user.avatar_url.as_deref())?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE users SET name = ?1, phone = ?2, avatar_url = ?3, about = ?4, link_previews_enabled = ?5 WHERE id = ?6",
        (
            &user.name,
            &user.phone,
            &user.avatar_url,
            &user.about,
            if user.link_previews_enabled { 1 } else { 0 },
            &user.id,
        ),
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn set_phone_number(
    app: AppHandle,
    db: State<'_, Database>,
    phone: String,
) -> Result<User, String> {
    // Validate and normalize phone number
    let trimmed_id = validate_phone_id(&phone)?;

    let mut conn = db.0.lock().map_err(|e| e.to_string())?;
    let current_id: String = conn
        .query_row("SELECT id FROM users WHERE is_self = 1", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    if trimmed_id == current_id {
        return conn
            .query_row(
                "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online, link_previews_enabled FROM users WHERE id = ?1",
                [&current_id],
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
                        link_previews_enabled: row.get::<_, i32>(8).unwrap_or(1) == 1,
                    })
                },
            )
            .map_err(|e| e.to_string());
    }

    let id_exists: bool = conn
        .query_row("SELECT 1 FROM users WHERE id = ?1", [&trimmed_id], |_| Ok(true))
        .unwrap_or(false);
    if id_exists {
        return Err("Phone number already registered".to_string());
    }

    let public_key_exists: bool = conn
        .query_row(
            "SELECT 1 FROM public_keys WHERE user_id = ?1",
            [&trimmed_id],
            |_| Ok(true),
        )
        .unwrap_or(false);
    if public_key_exists {
        return Err("Phone number already registered".to_string());
    }

    let has_participation: bool = conn
        .query_row(
            "SELECT 1 FROM chat_participants WHERE user_id = ?1 LIMIT 1",
            [&current_id],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or(false);

    let has_messages: bool = conn
        .query_row(
            "SELECT 1 FROM messages WHERE sender_id = ?1 LIMIT 1",
            [&current_id],
            |_| Ok(true),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or(false);

    if has_participation || has_messages {
        return Err("Cannot change ID after chats exist".to_string());
    }

    let old_private_key = storage::load_private_key(&current_id)?;
    if let Some(ref key_bytes) = old_private_key {
        storage::store_private_key(&trimmed_id, key_bytes)?;
    }

    // Disable FK checks for the transaction (updating PK with FK references)
    conn.execute("PRAGMA foreign_keys = OFF", [])
        .map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let update_result = (|| -> Result<(), String> {
        tx.execute(
            "UPDATE users SET id = ?1 WHERE id = ?2",
            (&trimmed_id, &current_id),
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE chat_participants SET user_id = ?1 WHERE user_id = ?2",
            (&trimmed_id, &current_id),
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE messages SET sender_id = ?1 WHERE sender_id = ?2",
            (&trimmed_id, &current_id),
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE public_keys SET user_id = ?1 WHERE user_id = ?2",
            (&trimmed_id, &current_id),
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    })();

    // Re-enable FK checks
    let _ = conn.execute("PRAGMA foreign_keys = ON", []);

    if let Err(err) = update_result {
        if old_private_key.is_some() {
            let _ = storage::delete_private_key(&trimmed_id);
        }
        return Err(err);
    }

    write_identity_file(&app, &trimmed_id)?;

    if old_private_key.is_some() {
        let _ = storage::delete_private_key(&current_id);
    }

    conn.query_row(
        "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online, link_previews_enabled FROM users WHERE id = ?1",
        [&trimmed_id],
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
                link_previews_enabled: row.get::<_, i32>(8).unwrap_or(1) == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_contacts(db: State<'_, Database>) -> Result<Vec<User>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online, link_previews_enabled
             FROM users
             WHERE is_self = 0
             ORDER BY COALESCE(display_name, name)",
        )
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                display_name: row.get(2)?,
                phone: row.get(3)?,
                avatar_url: row.get(4)?,
                about: row.get(5)?,
                last_seen: row.get(6)?,
                is_online: row.get::<_, i32>(7)? == 1,
                link_previews_enabled: row.get::<_, i32>(8).unwrap_or(1) == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(users)
}

#[tauri::command]
pub fn add_contact(
    db: State<'_, Database>,
    id: String,
    name: String,
    phone: Option<String>,
) -> Result<User, String> {
    // Validate and normalize contact ID (accepts phone numbers with + prefix)
    let normalized_id = validate_phone_id(&id)?;
    validate_user_name(&name)?;
    validate_phone(phone.as_deref())?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();

    // Check if contact already exists
    let exists: bool = conn
        .query_row("SELECT 1 FROM users WHERE id = ?1", [&normalized_id], |_| Ok(true))
        .unwrap_or(false);

    if exists {
        return Err("Contact already exists".to_string());
    }

    conn.execute(
        "INSERT INTO users (id, name, phone, avatar_url, about, last_seen, is_online, is_self)
         VALUES (?1, ?2, ?3, '', 'Hey there! I am using Pulse', ?4, 0, 0)",
        (&normalized_id, &name, &phone, now),
    )
    .map_err(|e| e.to_string())?;

    Ok(User {
        id: normalized_id,
        name,
        display_name: None,
        phone,
        avatar_url: Some("".to_string()),
        about: Some("Hey there! I am using Pulse".to_string()),
        last_seen: Some(now),
        is_online: false,
        link_previews_enabled: true,
    })
}

/// Save a contact with a custom display name (alias)
/// This sets the display_name field which overrides the original name in the UI
#[tauri::command]
pub fn save_contact(
    db: State<'_, Database>,
    user_id: String,
    display_name: String,
) -> Result<User, String> {
    validate_phone_id(&user_id)?;
    validate_user_name(&display_name)?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Check if user exists
    let exists: bool = conn
        .query_row("SELECT 1 FROM users WHERE id = ?1", [&user_id], |_| Ok(true))
        .unwrap_or(false);

    if !exists {
        return Err("User not found".to_string());
    }

    // Update display_name
    conn.execute(
        "UPDATE users SET display_name = ?1 WHERE id = ?2",
        (&display_name, &user_id),
    )
    .map_err(|e| e.to_string())?;

    // Return updated user
    conn.query_row(
        "SELECT id, name, display_name, phone, avatar_url, about, last_seen, is_online, link_previews_enabled FROM users WHERE id = ?1",
        [&user_id],
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
                link_previews_enabled: row.get::<_, i32>(8).unwrap_or(1) == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

const MAX_AVATAR_SIZE: usize = 512 * 1024; // 512KB max
const ALLOWED_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

#[tauri::command]
pub fn upload_avatar(app: AppHandle, source_path: String) -> Result<String, String> {
    // Validate source path exists
    let source = Path::new(&source_path);
    if !source.exists() {
        return Err("Source file does not exist".to_string());
    }

    // Validate file extension
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("Invalid file extension")?;

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(format!(
            "Invalid file type. Allowed: {}",
            ALLOWED_EXTENSIONS.join(", ")
        ));
    }

    // Validate file size
    let metadata = fs::metadata(source).map_err(|e| e.to_string())?;
    if metadata.len() as usize > MAX_AVATAR_SIZE {
        return Err(format!(
            "File too large. Maximum size: {}KB",
            MAX_AVATAR_SIZE / 1024
        ));
    }

    // Create avatars directory
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let avatars_dir = app_dir.join("avatars");
    fs::create_dir_all(&avatars_dir).map_err(|e| e.to_string())?;

    // Generate unique filename
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let dest_path = avatars_dir.join(&filename);

    // Copy file
    fs::copy(source, &dest_path).map_err(|e| e.to_string())?;

    // Return the destination path as string
    dest_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or("Invalid path encoding".to_string())
}

#[tauri::command]
pub fn save_peer_avatar(
    app: AppHandle,
    user_id: String,
    avatar_data: String,
) -> Result<String, String> {
    // Validate user ID (can be phone number with + prefix)
    validate_phone_id(&user_id)?;

    // Decode base64
    let bytes = BASE64
        .decode(&avatar_data)
        .map_err(|e| format!("Invalid base64 data: {}", e))?;

    // Validate size
    if bytes.len() > MAX_AVATAR_SIZE {
        return Err(format!(
            "Avatar too large. Maximum size: {}KB",
            MAX_AVATAR_SIZE / 1024
        ));
    }

    // Create avatars directory
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let avatars_dir = app_dir.join("avatars");
    fs::create_dir_all(&avatars_dir).map_err(|e| e.to_string())?;

    // Detect image format from magic bytes and save with appropriate extension
    let ext = detect_image_format(&bytes).unwrap_or("png");
    let filename = format!("{}.{}", user_id, ext);
    let dest_path = avatars_dir.join(&filename);

    // Write file
    fs::write(&dest_path, &bytes).map_err(|e| e.to_string())?;

    dest_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or("Invalid path encoding".to_string())
}

fn detect_image_format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 4 {
        return None;
    }
    // PNG: 89 50 4E 47
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return Some("png");
    }
    // JPEG: FF D8 FF
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("jpg");
    }
    // WebP: RIFF....WEBP
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }
    None
}
