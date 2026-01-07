use super::storage;
use super::types::{EncryptedMessage, IdentityInfo, KeyPair, SerializableKeyPair};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use rusqlite::Connection;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Mutex;
use x25519_dalek::{PublicKey, StaticSecret};

/// Manages encryption keys and sessions with persistent storage
pub struct CryptoManager {
    identity_key: Mutex<Option<KeyPair>>,
    session_keys: Mutex<HashMap<String, [u8; 32]>>,
    /// Cache of peer public keys (loaded from DB)
    peer_public_keys: Mutex<HashMap<String, [u8; 32]>>,
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoManager {
    pub fn new() -> Self {
        Self {
            identity_key: Mutex::new(None),
            session_keys: Mutex::new(HashMap::new()),
            peer_public_keys: Mutex::new(HashMap::new()),
        }
    }

    /// Generate a new X25519 key pair
    pub fn generate_identity_key(&self) -> Result<SerializableKeyPair, String> {
        let mut rng = rand::thread_rng();
        let private_key = StaticSecret::random_from_rng(&mut rng);
        let public_key = PublicKey::from(&private_key);

        let serializable = SerializableKeyPair {
            public_key: public_key.as_bytes().to_vec(),
            private_key: private_key.as_bytes().to_vec(),
        };

        *self.identity_key.lock().unwrap() = Some(KeyPair {
            public_key,
            private_key,
        });

        Ok(serializable)
    }

    /// Get the current identity key or generate one
    pub fn get_identity_key(&self) -> Result<SerializableKeyPair, String> {
        let guard = self.identity_key.lock().unwrap();
        if let Some(ref key) = *guard {
            Ok(SerializableKeyPair {
                public_key: key.public_key.as_bytes().to_vec(),
                private_key: key.private_key.as_bytes().to_vec(),
            })
        } else {
            drop(guard);
            self.generate_identity_key()
        }
    }

    /// Derive a shared secret using X25519 and HKDF
    fn derive_session_key(
        &self,
        their_public_key: &[u8],
        chat_id: &str,
    ) -> Result<[u8; 32], String> {
        let guard = self.identity_key.lock().unwrap();
        let keypair = guard.as_ref().ok_or("No identity key")?;

        // Convert their public key
        let their_key: [u8; 32] = their_public_key
            .try_into()
            .map_err(|_| "Invalid public key length")?;
        let their_public = PublicKey::from(their_key);

        // Perform X25519 key exchange
        let shared_secret = keypair.private_key.diffie_hellman(&their_public);

        // Derive session key using HKDF
        let hk = Hkdf::<Sha256>::new(Some(chat_id.as_bytes()), shared_secret.as_bytes());
        let mut session_key = [0u8; 32];
        hk.expand(b"pulse-e2e-session", &mut session_key)
            .map_err(|_| "HKDF expansion failed")?;

        Ok(session_key)
    }

    /// Initialize a session with another user
    pub fn init_session(&self, their_public_key: &[u8], chat_id: &str) -> Result<(), String> {
        let session_key = self.derive_session_key(their_public_key, chat_id)?;
        self.session_keys
            .lock()
            .unwrap()
            .insert(chat_id.to_string(), session_key);
        Ok(())
    }

    /// Encrypt a message using AES-256-GCM
    pub fn encrypt(&self, plaintext: &str, chat_id: &str) -> Result<EncryptedMessage, String> {
        let session_key = {
            let keys = self.session_keys.lock().unwrap();
            keys.get(chat_id).copied()
        };

        let session_key =
            session_key.ok_or("No session key for this chat. Initialize session first.")?;

        // Get sender's public key
        let identity = self.get_identity_key()?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher and encrypt
        let cipher =
            Aes256Gcm::new_from_slice(&session_key).map_err(|_| "Failed to create cipher")?;

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| "Encryption failed")?;

        Ok(EncryptedMessage {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            sender_public_key: identity.public_key,
        })
    }

    /// Decrypt a message using AES-256-GCM
    pub fn decrypt(&self, encrypted: &EncryptedMessage, chat_id: &str) -> Result<String, String> {
        let session_key = {
            let keys = self.session_keys.lock().unwrap();
            keys.get(chat_id).copied()
        };

        let session_key = session_key.ok_or("No session key for this chat")?;

        // Get nonce
        let nonce_bytes: [u8; 12] = encrypted
            .nonce
            .clone()
            .try_into()
            .map_err(|_| "Invalid nonce length")?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher and decrypt
        let cipher =
            Aes256Gcm::new_from_slice(&session_key).map_err(|_| "Failed to create cipher")?;

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| "Decryption failed - message may be tampered")?;

        String::from_utf8(plaintext).map_err(|_| "Invalid UTF-8 in decrypted message".to_string())
    }

    /// Check if a session exists for a chat
    pub fn has_session(&self, chat_id: &str) -> bool {
        self.session_keys.lock().unwrap().contains_key(chat_id)
    }

    // ============= Persistent Key Management =============

    /// Initialize identity from persistent storage or generate new keys
    /// This should be called once during app startup
    pub fn init_identity(&self, conn: &Connection, user_id: &str) -> Result<IdentityInfo, String> {
        // Step 1: Try to load existing keys
        let stored_public_key = storage::load_public_key(conn, user_id)?;
        let stored_private_key = storage::load_private_key(user_id)?;

        match (stored_public_key, stored_private_key) {
            // Both keys exist - restore identity
            (Some(public_bytes), Some(private_bytes)) => {
                let public_key: [u8; 32] = public_bytes
                    .try_into()
                    .map_err(|_| "Invalid public key length")?;

                let private_key = StaticSecret::from(private_bytes);
                let public = PublicKey::from(public_key);

                *self.identity_key.lock().unwrap() = Some(KeyPair {
                    public_key: public,
                    private_key,
                });

                Ok(IdentityInfo {
                    user_id: user_id.to_string(),
                    public_key_hex: hex::encode(public_key),
                    is_new: false,
                })
            }
            // Keys missing or mismatched - generate new
            _ => {
                let keypair = self.generate_identity_key()?;

                // Store private key in OS keyring
                storage::store_private_key(user_id, &keypair.private_key)?;

                // Store public key in database
                storage::store_public_key(conn, user_id, &keypair.public_key, "identity")?;

                Ok(IdentityInfo {
                    user_id: user_id.to_string(),
                    public_key_hex: hex::encode(&keypair.public_key),
                    is_new: true,
                })
            }
        }
    }

    /// Store a peer's public key in database and cache
    pub fn store_peer_public_key(
        &self,
        conn: &Connection,
        peer_user_id: &str,
        public_key: &[u8],
    ) -> Result<(), String> {
        // Validate key length
        if public_key.len() != 32 {
            return Err("Invalid public key length".to_string());
        }

        // Store in database
        storage::store_public_key(conn, peer_user_id, public_key, "peer")?;

        // Update cache
        let key: [u8; 32] = public_key.try_into().unwrap();
        self.peer_public_keys
            .lock()
            .unwrap()
            .insert(peer_user_id.to_string(), key);

        Ok(())
    }

    /// Get a peer's public key (from cache or database)
    pub fn get_peer_public_key(
        &self,
        conn: &Connection,
        peer_user_id: &str,
    ) -> Result<Option<[u8; 32]>, String> {
        // Check cache first
        {
            let cache = self.peer_public_keys.lock().unwrap();
            if let Some(&key) = cache.get(peer_user_id) {
                return Ok(Some(key));
            }
        }

        // Load from database
        if let Some(key_bytes) = storage::load_public_key(conn, peer_user_id)? {
            let key: [u8; 32] = key_bytes
                .try_into()
                .map_err(|_| "Invalid public key length")?;

            // Update cache
            self.peer_public_keys
                .lock()
                .unwrap()
                .insert(peer_user_id.to_string(), key);

            return Ok(Some(key));
        }

        Ok(None)
    }

    /// Load all peer public keys into cache (call during init)
    pub fn load_peer_keys_from_db(&self, conn: &Connection) -> Result<(), String> {
        let keys = storage::load_all_peer_keys(conn)?;
        let mut cache = self.peer_public_keys.lock().unwrap();

        for (user_id, key_bytes) in keys {
            if let Ok(key) = key_bytes.try_into() {
                cache.insert(user_id, key);
            }
        }

        Ok(())
    }

    /// Ensure a session exists for a chat (auto-derive if peer key available)
    pub fn ensure_session(
        &self,
        conn: &Connection,
        peer_user_id: &str,
        chat_id: &str,
    ) -> Result<bool, String> {
        // Check if session already exists
        if self.has_session(chat_id) {
            return Ok(true);
        }

        // Try to get peer's public key
        if let Some(peer_public_key) = self.get_peer_public_key(conn, peer_user_id)? {
            self.init_session(&peer_public_key, chat_id)?;
            Ok(true)
        } else {
            Ok(false) // No public key available
        }
    }
}
