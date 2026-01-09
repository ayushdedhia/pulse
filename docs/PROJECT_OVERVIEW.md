# Pulse - Project Overview

A WhatsApp-like desktop chat application built with Tauri (Rust backend) + React (TypeScript frontend).

## Tech Stack
- **Frontend**: React 18 + TypeScript + Tailwind CSS
- **Backend**: Tauri 2.0 (Rust)
- **Build Tool**: Vite
- **State Management**: Zustand (split stores)
- **Database**: SQLite (local via rusqlite)
- **Real-time**: WebSocket (tokio-tungstenite)
- **Encryption**: X25519 + AES-256-GCM (E2E)
- **Icons**: Lucide React
- **Fonts**: Inter (UI), JetBrains Mono (monospace for IDs/IPs)

## Target Features (WhatsApp Parity)

### UI Components
- [x] Three-panel layout (sidebar, chat list, chat window)
- [x] Contact/Chat list with search
- [x] Chat bubbles (sent/received styling with CSS tails)
- [x] Message status (sent, delivered, read - double ticks)
- [x] User avatars with gradient fallbacks
- [x] Message timestamps
- [x] Typing indicators (animated)
- [x] Emoji picker
- [x] File/image attachment UI (buttons ready)
- [ ] Voice message support
- [x] Dark/Light theme toggle
- [x] Resizable chat list panel (280-500px)
- [x] Smooth animations (fade-in, slide-up, scale-in)
- [ ] Splash screen

### Core Features
- [ ] User authentication (phone/email)
- [x] 1-on-1 messaging (UI ready, backend commands ready)
- [x] Group chats (UI ready)
- [x] Message encryption (E2E) - X25519 key exchange + AES-256-GCM
- [ ] Push notifications
- [x] Message search (backend command ready)
- [ ] Media gallery
- [x] Contact management (backend commands ready)
- [x] Profile settings modal

### Backend (Tauri/Rust)
- [x] SQLite database initialization
- [x] Database schema (users, chats, chat_participants, messages)
- [x] Seed data for testing
- [x] IPC Commands: get_chats, get_messages, send_message, mark_as_read, create_chat, get_user, get_current_user, update_user, search_messages, get_contacts
- [x] Real-time message delivery (WebSocket on port 9001)
- [x] Typing indicators via WebSocket
- [x] User presence (online/offline/last seen)
- [x] E2E Crypto Commands: generate_keys, get_public_key, init_chat_session, encrypt_message, decrypt_message, has_chat_session
- [x] Persistent key storage: init_identity, store_peer_key, get_peer_key, ensure_chat_session
- [ ] File upload/storage

## Design Reference
- Primary green: #00A884 (WhatsApp teal)
- Dark mode background: #111B21
- Dark mode panels: #202C33
- Dark mode hover: #2A3942
- Light mode: #FFFFFF, #F0F2F5
- Fonts: Inter (UI), JetBrains Mono (IDs, IPs, phone numbers)
- Animations: fadeIn, slideUp, scaleIn, typingBounce
- Custom titlebar with native window controls (decorations: false)

## Current Status
- Full UI prototype complete with WhatsApp-like styling
- SQLite backend with seed data
- All major components implemented
- Theme switching (dark/light) working
- Resizable chat list panel
- **Frontend fully connected to backend** (Tauri IPC)
  - Chats load from SQLite on app start
  - Messages load when selecting a chat
  - Sending messages persists to database
  - Mark as read on chat open
- **WebSocket real-time messaging**
  - Server runs on localhost:9001
  - Typing indicators broadcast
  - User presence (online/offline)
  - Message sync across clients
- **E2E Encryption implemented**
  - X25519 Diffie-Hellman key exchange
  - AES-256-GCM authenticated encryption
  - HKDF key derivation
  - Per-chat session keys
  - **Persistent key storage** (identity keys survive app restarts)
    - Private keys stored in OS Keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
    - Public keys stored in SQLite (`public_keys` table)
    - Peer public keys cached and persisted for automatic session re-derivation
    - Uses `keyring` crate for cross-platform secure storage
- **Multi-instance P2P messaging working**
  - First instance becomes WebSocket server (port 9001)
  - Second instance auto-connects as client relay
  - Messages sync in real-time between instances
  - Deterministic chat IDs (both parties have same chat ID)
  - Auto-creates sender as contact when receiving message from unknown user
  - Profile modal shows copyable Pulse ID for sharing
  - Correct message alignment (own messages on right, others on left)
  - Typing indicators filtered to not show self
  - Unique default usernames (User + UUID prefix)
- **Codebase refactored for modularity**
  - Backend: models/, commands/, websocket/, crypto/, utils/ modules
  - Frontend: services/ layer, split Zustand stores
- **Custom window titlebar**
  - Native decorations disabled (decorations: false in tauri.conf.json)
  - Custom Titlebar component with minimize/maximize/close buttons
  - Window dragging via startDragging() API
  - Double-click to maximize
  - Tauri 2.0 capabilities/permissions configured in capabilities/default.json
- **LAN networking support**
  - WebSocket server binds to 0.0.0.0:9001 for LAN access
  - Network modal shows local IP address
  - Manual peer connection by IP address
  - Uses `local-ip-address` crate for IP detection
- **UI polish**
  - JetBrains Mono font for IDs, IPs, phone numbers, unread counts
  - Attachment menu closes on outside click
  - Message input vertically centered
- **Message delivery receipts (double ticks)**
  - Single tick (✓): Message sent from device
  - Double gray tick (✓✓): Message delivered to recipient's device
  - Double blue tick (✓✓): Message read by recipient
  - Real-time status updates via WebSocket (DeliveryReceipt, ReadReceipt)
  - Auto-sends read receipts when window gains focus
  - Optimized store updates using addMessage instead of full reload

## Next Steps

### Security Enhancements
- [x] Implement tracing for robust logging (tracing + tracing-subscriber)
- [ ] Add rate limiting to WebSocket server
- [ ] Implement forward secrecy (Signal protocol ratcheting)
- [ ] Key verification UI (safety numbers/QR codes)
- [ ] Add zeroize for keys in memory

### Features
- [ ] Voice message recording
- [ ] File/image upload and preview
- [x] Store encryption keys persistently
- [ ] Auto network discovery for LAN peers (mDNS/broadcast)
- [ ] Edit contact details (name, phone) from chat header or contact list
- [x] Save profile changes (name, about) in ProfileModal

## Notes
- Offline-first architecture (local SQLite)
- Networking/cloud sync to be added later
- WebSocket binds to 0.0.0.0:9001 (accessible on LAN)
- Multi-instance works on same machine via separate Tauri configs
- Custom titlebar requires capabilities/default.json for window permissions in Tauri 2.0
