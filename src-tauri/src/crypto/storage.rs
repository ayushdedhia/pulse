use keyring::Entry;
use rusqlite::Connection;

const KEYRING_SERVICE: &str = "pulse-chat";
const KEYRING_IDENTITY_PREFIX: &str = "identity-key-";

/// Store private key in OS keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
pub fn store_private_key(user_id: &str, private_key: &[u8]) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, &format!("{}{}", KEYRING_IDENTITY_PREFIX, user_id))
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

    // Store as hex string (keyring stores strings)
    let key_hex = hex::encode(private_key);
    entry
        .set_password(&key_hex)
        .map_err(|e| format!("Failed to store private key: {}", e))?;

    Ok(())
}

/// Retrieve private key from OS keyring
pub fn load_private_key(user_id: &str) -> Result<Option<[u8; 32]>, String> {
    let entry = Entry::new(KEYRING_SERVICE, &format!("{}{}", KEYRING_IDENTITY_PREFIX, user_id))
        .map_err(|e| format!("Failed to access keyring: {}", e))?;

    match entry.get_password() {
        Ok(key_hex) => {
            let key_bytes = hex::decode(&key_hex)
                .map_err(|e| format!("Invalid key format in keyring: {}", e))?;
            let key: [u8; 32] = key_bytes
                .try_into()
                .map_err(|_| "Invalid key length in keyring".to_string())?;
            Ok(Some(key))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to retrieve private key: {}", e)),
    }
}

/// Delete private key from OS keyring (for key rotation/deletion)
#[allow(dead_code)]
pub fn delete_private_key(user_id: &str) -> Result<(), String> {
    let entry = Entry::new(KEYRING_SERVICE, &format!("{}{}", KEYRING_IDENTITY_PREFIX, user_id))
        .map_err(|e| format!("Failed to access keyring: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(format!("Failed to delete private key: {}", e)),
    }
}

/// Store public key in SQLite database
pub fn store_public_key(
    conn: &Connection,
    user_id: &str,
    public_key: &[u8],
    key_type: &str,
) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp_millis();

    conn.execute(
        "INSERT INTO public_keys (user_id, public_key, key_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(user_id) DO UPDATE SET
         public_key = ?2, updated_at = ?5",
        (user_id, public_key, key_type, now, now),
    )
    .map_err(|e| format!("Failed to store public key: {}", e))?;

    Ok(())
}

/// Load public key from SQLite database
pub fn load_public_key(conn: &Connection, user_id: &str) -> Result<Option<Vec<u8>>, String> {
    match conn.query_row(
        "SELECT public_key FROM public_keys WHERE user_id = ?1",
        [user_id],
        |row| row.get::<_, Vec<u8>>(0),
    ) {
        Ok(key) => Ok(Some(key)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to load public key: {}", e)),
    }
}

/// Load all peer public keys from database (for cache initialization)
pub fn load_all_peer_keys(conn: &Connection) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut stmt = conn
        .prepare("SELECT user_id, public_key FROM public_keys WHERE key_type = 'peer'")
        .map_err(|e| e.to_string())?;

    let keys = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(keys)
}
