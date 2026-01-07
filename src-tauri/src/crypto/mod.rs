mod manager;
mod types;

pub use manager::CryptoManager;
pub use types::EncryptedMessage;

use std::sync::OnceLock;

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
