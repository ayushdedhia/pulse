use crate::db::Database;
use crate::models::User;
use tauri::State;

#[tauri::command]
pub fn get_user(db: State<'_, Database>, user_id: String) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, name, phone, avatar_url, about, last_seen, is_online FROM users WHERE id = ?1",
        [&user_id],
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
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_user(db: State<'_, Database>) -> Result<User, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.query_row(
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
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_user(db: State<'_, Database>, user: User) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE users SET name = ?1, avatar_url = ?2, about = ?3 WHERE id = ?4",
        (&user.name, &user.avatar_url, &user.about, &user.id),
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn get_contacts(db: State<'_, Database>) -> Result<Vec<User>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, phone, avatar_url, about, last_seen, is_online
             FROM users
             WHERE is_self = 0
             ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                phone: row.get(2)?,
                avatar_url: row.get(3)?,
                about: row.get(4)?,
                last_seen: row.get(5)?,
                is_online: row.get::<_, i32>(6)? == 1,
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
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp_millis();

    // Check if contact already exists
    let exists: bool = conn
        .query_row("SELECT 1 FROM users WHERE id = ?1", [&id], |_| Ok(true))
        .unwrap_or(false);

    if exists {
        return Err("Contact already exists".to_string());
    }

    conn.execute(
        "INSERT INTO users (id, name, phone, avatar_url, about, last_seen, is_online, is_self)
         VALUES (?1, ?2, ?3, '', 'Hey there! I am using Pulse', ?4, 0, 0)",
        (&id, &name, &phone, now),
    )
    .map_err(|e| e.to_string())?;

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
