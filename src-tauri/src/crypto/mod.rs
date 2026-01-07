mod manager;
mod storage;
mod types;

pub use manager::CryptoManager;
pub use types::EncryptedMessage;
pub use types::IdentityInfo;

use crate::db::Database;
use std::sync::OnceLock;
use tauri::State;

// Global crypto manager instance
static CRYPTO_MANAGER: OnceLock<CryptoManager> = OnceLock::new();

pub fn get_crypto_manager() -> &'static CryptoManager {
    CRYPTO_MANAGER.get_or_init(CryptoManager::new)
}

// ============= Tauri Commands =============

#[tauri::command]
pub fn generate_keys() -> Result<String, String> {
    let manager = get_crypto_manager();
    let keypair = manager.generate_identity_key()?;
    Ok(hex::encode(&keypair.public_key))
}

#[tauri::command]
pub fn get_public_key() -> Result<String, String> {
    let manager = get_crypto_manager();
    let keypair = manager.get_identity_key()?;
    Ok(hex::encode(&keypair.public_key))
}

#[tauri::command]
pub fn init_chat_session(their_public_key: String, chat_id: String) -> Result<bool, String> {
    let manager = get_crypto_manager();
    let key_bytes = hex::decode(&their_public_key).map_err(|e| e.to_string())?;
    manager.init_session(&key_bytes, &chat_id)?;
    Ok(true)
}

#[tauri::command]
pub fn encrypt_message(plaintext: String, chat_id: String) -> Result<String, String> {
    let manager = get_crypto_manager();
    let encrypted = manager.encrypt(&plaintext, &chat_id)?;
    serde_json::to_string(&encrypted).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decrypt_message(encrypted_json: String, chat_id: String) -> Result<String, String> {
    let manager = get_crypto_manager();
    let encrypted: EncryptedMessage =
        serde_json::from_str(&encrypted_json).map_err(|e| e.to_string())?;
    manager.decrypt(&encrypted, &chat_id)
}

#[tauri::command]
pub fn has_chat_session(chat_id: String) -> Result<bool, String> {
    let manager = get_crypto_manager();
    Ok(manager.has_session(&chat_id))
}

// ============= Persistent Key Commands =============

/// Initialize identity keys from persistent storage
/// Called once during app startup
#[tauri::command]
pub fn init_identity(db: State<'_, Database>) -> Result<IdentityInfo, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Get current user ID
    let user_id: String = conn
        .query_row("SELECT id FROM users WHERE is_self = 1", [], |row| row.get(0))
        .map_err(|e| format!("Failed to get user ID: {}", e))?;

    let manager = get_crypto_manager();

    // Initialize identity (load or generate)
    let info = manager.init_identity(&conn, &user_id)?;

    // Also load all peer public keys into cache
    manager.load_peer_keys_from_db(&conn)?;

    Ok(info)
}

/// Store a peer's public key (received during key exchange)
#[tauri::command]
pub fn store_peer_key(
    db: State<'_, Database>,
    peer_user_id: String,
    public_key_hex: String,
) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let key_bytes = hex::decode(&public_key_hex).map_err(|e| e.to_string())?;

    get_crypto_manager().store_peer_public_key(&conn, &peer_user_id, &key_bytes)?;
    Ok(true)
}

/// Get a peer's public key from storage
#[tauri::command]
pub fn get_peer_key(
    db: State<'_, Database>,
    peer_user_id: String,
) -> Result<Option<String>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    if let Some(key) = get_crypto_manager().get_peer_public_key(&conn, &peer_user_id)? {
        Ok(Some(hex::encode(key)))
    } else {
        Ok(None)
    }
}

/// Ensure a session exists for a chat (auto-derives if peer key available)
#[tauri::command]
pub fn ensure_chat_session(
    db: State<'_, Database>,
    peer_user_id: String,
    chat_id: String,
) -> Result<bool, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    get_crypto_manager().ensure_session(&conn, &peer_user_id, &chat_id)
}
