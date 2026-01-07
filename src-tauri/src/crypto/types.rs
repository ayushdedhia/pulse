use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};

/// Serializable key pair for storage/transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

/// Encrypted message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub sender_public_key: Vec<u8>,
}

/// Internal key pair for X25519 key exchange
pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: StaticSecret,
}

/// Result of initializing identity from persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityInfo {
    pub user_id: String,
    pub public_key_hex: String,
    pub is_new: bool,
}
