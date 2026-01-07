use super::types::{EncryptedMessage, KeyPair, SerializableKeyPair};
use aes_gcm::{
   aead::{Aead, KeyInit},
   Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Mutex;
use x25519_dalek::{PublicKey, StaticSecret};

/// Manages encryption keys and sessions
pub struct CryptoManager {
   identity_key: Mutex<Option<KeyPair>>,
   session_keys: Mutex<HashMap<String, [u8; 32]>>,
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
      self
         .session_keys
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
}
