# Pulse - Security Guide

## Security Rules (Tauri / Crypto / Networking)
- Validate all inputs in Rust commands (never trust frontend).
- Prevent path traversal for any file path inputs.
- Never log secrets, keys, decrypted messages, or identity private material.
- Commands must return structured errors; no panics.
- Shell commands must be allowlisted and sanitized — never execute raw user input.
- WebSocket messages must be validated and rate-limited when possible.

## E2E Encryption Implementation

### Cryptographic Primitives
- **Key Exchange**: X25519 Diffie-Hellman
- **Symmetric Encryption**: AES-256-GCM (authenticated encryption)
- **Key Derivation**: HKDF
- **Per-chat session keys**: Each chat has its own derived session key

### Crypto Commands (Tauri IPC)
- `generate_keys` - Generate new X25519 keypair
- `get_public_key` - Get own public key for sharing
- `init_chat_session` - Initialize E2E session with peer
- `encrypt_message` - Encrypt message with session key
- `decrypt_message` - Decrypt received message
- `has_chat_session` - Check if session exists for chat

### Persistent Key Storage
- `init_identity` - Initialize or load identity keypair
- `store_peer_key` - Store peer's public key
- `get_peer_key` - Retrieve stored peer public key
- `ensure_chat_session` - Ensure session is established (auto-derives if peer key available)

### Key Storage Architecture
- **Private keys**: Stored in OS Keyring
  - Windows: Credential Manager
  - macOS: Keychain
  - Linux: Secret Service (libsecret)
- **Public keys**: Stored in SQLite (`public_keys` table)
- **Peer public keys**: Cached and persisted for automatic session re-derivation
- **Implementation**: Uses `keyring` crate for cross-platform secure storage

### Crypto Module Structure
```
src-tauri/src/crypto/
├── mod.rs        # Re-exports + Tauri commands
├── manager.rs    # CryptoManager struct with persistent storage
├── storage.rs    # OS Keyring + SQLite key storage
└── types.rs      # SerializableKeyPair, EncryptedMessage, IdentityInfo
```

## WebSocket Security

### Current Implementation
- Server binds to 0.0.0.0:9001 (LAN accessible)
- First instance becomes server, others connect as clients
- Messages sync in real-time between instances

### Security Considerations
- WebSocket messages must be validated
- Rate limiting should be implemented (TODO)
- Typing indicators and presence info broadcast to connected peers

## Security Roadmap

### Implemented
- [x] Tracing for robust logging (tracing + tracing-subscriber)
- [x] Persistent key storage (identity keys survive app restarts)

### Planned Enhancements
- [ ] Add rate limiting to WebSocket server
- [ ] Implement forward secrecy (Signal protocol ratcheting)
- [ ] Key verification UI (safety numbers/QR codes)
- [ ] Add zeroize for keys in memory
