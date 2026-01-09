# Pulse - Backend Guide

## Tech Stack
- **Framework**: Tauri 2.0 (Rust)
- **Database**: SQLite (local via rusqlite)
- **Real-time**: WebSocket (tokio-tungstenite)
- **Encryption**: X25519 + AES-256-GCM (E2E)
- **Key Storage**: keyring crate (cross-platform secure storage)

## Backend Structure
```
src-tauri/src/
├── main.rs                    # Tauri entry point
├── lib.rs                     # App setup + command registration
├── db.rs                      # SQLite setup + seed data
├── models/                    # Data structures
│   ├── mod.rs                # Re-exports
│   ├── user.rs               # User struct
│   ├── chat.rs               # Chat struct
│   └── message.rs            # Message struct
├── commands/                  # IPC handlers by domain
│   ├── mod.rs                # Re-exports all commands
│   ├── user.rs               # get_user, get_current_user, update_user, get_contacts, add_contact
│   ├── chat.rs               # get_chats, create_chat
│   ├── message.rs            # get_messages, send_message, mark_as_read, search_messages, receive_message
│   └── websocket.rs          # broadcast_message, get_ws_port, connect_to_peer, get_network_status
├── websocket/                 # WebSocket server
│   ├── mod.rs                # Re-exports + init_websocket_server
│   ├── server.rs             # WebSocketServer struct + NetworkStatus, PeerInfo
│   ├── handlers.rs           # Connection handlers
│   └── messages.rs           # WsMessage enum
├── capabilities/              # Tauri 2.0 permissions
│   └── default.json          # Window permissions (close, minimize, maximize, drag)
├── crypto/                    # E2E encryption
│   ├── mod.rs                # Re-exports + Tauri commands
│   ├── manager.rs            # CryptoManager struct with persistent storage
│   ├── storage.rs            # OS Keyring + SQLite key storage
│   └── types.rs              # SerializableKeyPair, EncryptedMessage, IdentityInfo
└── utils/                     # Shared helpers
    ├── mod.rs                # Re-exports
    └── helpers.rs            # get_self_id(), generate_deterministic_chat_id()
```

## Backend Module Organization
Commands are organized by domain in separate files:
- `commands::user` - User-related IPC handlers
- `commands::chat` - Chat management handlers
- `commands::message` - Message CRUD handlers
- `commands::websocket` - WebSocket-related handlers

Shared utilities extracted to `utils/helpers.rs`:
- `get_self_id(conn)` - Get current user's ID from database
- `generate_deterministic_chat_id(id1, id2)` - Create consistent chat IDs

## IPC Commands

### User Commands
- `get_user` - Get user by ID
- `get_current_user` - Get current logged-in user
- `update_user` - Update user profile
- `get_contacts` - Get all contacts
- `add_contact` - Add new contact

### Chat Commands
- `get_chats` - Get all chats for current user
- `create_chat` - Create new chat

### Message Commands
- `get_messages` - Get messages for a chat
- `send_message` - Send a new message
- `mark_as_read` - Mark messages as read
- `search_messages` - Search messages
- `receive_message` - Handle incoming message

### WebSocket Commands
- `broadcast_message` - Broadcast to connected peers
- `get_ws_port` - Get WebSocket server port
- `connect_to_peer` - Connect to peer by IP
- `get_network_status` - Get current network status

### Crypto Commands
- `generate_keys` - Generate new X25519 keypair
- `get_public_key` - Get own public key
- `init_chat_session` - Initialize E2E session
- `encrypt_message` - Encrypt message
- `decrypt_message` - Decrypt message
- `has_chat_session` - Check if session exists
- `init_identity` - Initialize/load identity
- `store_peer_key` - Store peer's public key
- `get_peer_key` - Get stored peer key
- `ensure_chat_session` - Ensure session established

## Database Schema
- `users` - User accounts
- `chats` - Chat conversations
- `chat_participants` - Chat membership
- `messages` - Message storage
- `public_keys` - Stored public keys for E2E

## Security Rules
- Validate all inputs in Rust commands (never trust frontend).
- Prevent path traversal for any file path inputs.
- Never log secrets, keys, decrypted messages, or identity private material.
- Commands must return structured errors; no panics.
- Shell commands must be allowlisted and sanitized — never execute raw user input.
- WebSocket messages must be validated and rate-limited when possible.

## Conventions
- Do NOT invent APIs, commands, types, or file paths.
- Always follow existing patterns (commands/ modules).
- If touching Rust commands, update the corresponding frontend service wrapper.
- If adding new commands, update registration + types + service exports.
- Do not rewrite existing modules unless required.
