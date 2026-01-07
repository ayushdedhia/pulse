# Pulse - WhatsApp Clone

## Project Overview
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
- [ ] File upload/storage

## Project Structure (Refactored)

### Frontend (src/)
```
src/
├── components/
│   ├── layout/
│   │   ├── AppLayout.tsx       # Main 3-panel layout with resize
│   │   ├── Titlebar.tsx        # Custom window titlebar (minimize/maximize/close)
│   │   ├── Sidebar.tsx         # Navigation sidebar
│   │   ├── ChatList.tsx        # Chat list with search/filters
│   │   └── ChatWindow.tsx      # Active chat view
│   ├── chat/
│   │   ├── ChatHeader.tsx      # Chat header with actions
│   │   ├── MessageList.tsx     # Scrollable message container
│   │   ├── MessageBubble.tsx   # Individual message styling
│   │   ├── MessageInput.tsx    # Input with emoji/attachments
│   │   ├── MessageStatus.tsx   # Tick indicators
│   │   ├── DateDivider.tsx     # Date separators
│   │   ├── TypingIndicator.tsx # Typing animation
│   │   └── EmojiPicker.tsx     # Emoji selector grid
│   ├── chat-list/
│   │   └── ChatListItem.tsx    # Individual chat preview
│   ├── common/
│   │   └── Avatar.tsx          # User avatar with fallback
│   └── modals/
│       ├── NewChatModal.tsx    # Create new chat
│       ├── ProfileModal.tsx    # User profile editor
│       └── NetworkModal.tsx    # LAN network settings
├── services/                   # API abstraction layer (NEW)
│   ├── index.ts               # Re-exports all services
│   ├── userService.ts         # User API calls
│   ├── chatService.ts         # Chat API calls
│   ├── messageService.ts      # Message API calls
│   └── websocketService.ts    # WebSocket API calls
├── store/                      # Zustand stores (REFACTORED)
│   ├── chatStore.ts           # Chat list + active chat state
│   ├── messageStore.ts        # Messages by chatId (NEW)
│   ├── userStore.ts           # Current user state (NEW)
│   └── uiStore.ts             # UI state (theme, modals)
├── utils/                      # Utility functions (NEW)
│   └── formatTime.ts          # Time formatting utilities
├── hooks/
│   ├── useWebSocket.ts        # WebSocket connection hook
│   └── useCrypto.ts           # E2E encryption hook
├── context/
│   └── WebSocketContext.tsx   # WebSocket provider
├── types/
│   └── index.ts               # TypeScript interfaces
├── App.tsx
├── main.tsx
└── index.css                  # Global styles + animations
```

### Backend (src-tauri/src/) - Modular Architecture
```
src-tauri/src/
├── main.rs                    # Tauri entry point
├── lib.rs                     # App setup + command registration
├── db.rs                      # SQLite setup + seed data
├── models/                    # Data structures (NEW)
│   ├── mod.rs                # Re-exports
│   ├── user.rs               # User struct
│   ├── chat.rs               # Chat struct
│   └── message.rs            # Message struct
├── commands/                  # IPC handlers by domain (NEW)
│   ├── mod.rs                # Re-exports all commands
│   ├── user.rs               # get_user, get_current_user, update_user, get_contacts, add_contact
│   ├── chat.rs               # get_chats, create_chat
│   ├── message.rs            # get_messages, send_message, mark_as_read, search_messages, receive_message
│   └── websocket.rs          # broadcast_message, get_ws_port, connect_to_peer, get_network_status
├── websocket/                 # WebSocket server (REFACTORED)
│   ├── mod.rs                # Re-exports + init_websocket_server
│   ├── server.rs             # WebSocketServer struct + NetworkStatus, PeerInfo
│   ├── handlers.rs           # Connection handlers
│   └── messages.rs           # WsMessage enum
├── capabilities/              # Tauri 2.0 permissions
│   └── default.json          # Window permissions (close, minimize, maximize, drag)
├── crypto/                    # E2E encryption (REFACTORED)
│   ├── mod.rs                # Re-exports + Tauri commands
│   ├── manager.rs            # CryptoManager struct
│   └── types.rs              # SerializableKeyPair, EncryptedMessage
└── utils/                     # Shared helpers (NEW)
    ├── mod.rs                # Re-exports
    └── helpers.rs            # get_self_id(), generate_deterministic_chat_id()
```

## Architecture Patterns

### Frontend Services Layer
All Tauri API calls are abstracted through services:
```typescript
// Instead of: invoke<User>("get_current_user")
// Use: userService.getCurrentUser()

import { userService, chatService, messageService } from "./services";
```

### Split Zustand Stores
State is split by domain for better separation of concerns:
- **userStore**: `currentUser`, `loadCurrentUser()`, `updateCurrentUser()`
- **chatStore**: `chats[]`, `activeChat`, `loadChats()`, `setActiveChat()`, `createChat()`
- **messageStore**: `messages{}`, `loadMessages()`, `sendMessage()`, `markAsRead()`
- **uiStore**: `theme`, `showNewChat`, `showProfile`, etc.

### Backend Module Organization
Commands are organized by domain in separate files:
- `commands::user` - User-related IPC handlers
- `commands::chat` - Chat management handlers
- `commands::message` - Message CRUD handlers
- `commands::websocket` - WebSocket-related handlers

Shared utilities extracted to `utils/helpers.rs`:
- `get_self_id(conn)` - Get current user's ID from database
- `generate_deterministic_chat_id(id1, id2)` - Create consistent chat IDs

## Design Reference
- Primary green: #00A884 (WhatsApp teal)
- Dark mode background: #111B21
- Dark mode panels: #202C33
- Dark mode hover: #2A3942
- Light mode: #FFFFFF, #F0F2F5
- Fonts: Inter (UI), JetBrains Mono (IDs, IPs, phone numbers)
- Animations: fadeIn, slideUp, scaleIn, typingBounce
- Custom titlebar with native window controls (decorations: false)

## Commands
```bash
npm run tauri dev      # Development
npm run tauri build    # Production build
npm run dev            # Vite dev server only
npx tsc --noEmit       # TypeScript type check
cargo check --manifest-path src-tauri/Cargo.toml  # Rust check
```

## VS Code Setup
The `.vscode/settings.json` configures rust-analyzer to suppress Tauri macro warnings.

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

## Multi-Instance Testing
```bash
# Terminal 1: Start main app (becomes WebSocket server)
npm run tauri dev

# Terminal 2: Start test instance (connects as client)
npm run tauri:test
```
Test instance uses separate database and target directory (`target-test/`).

## Next Steps
- [ ] Voice message recording
- [ ] File/image upload and preview
- [ ] Store encryption keys persistently
- [ ] Key verification UI (safety numbers)
- [ ] Auto network discovery for LAN peers (mDNS/broadcast)
- [ ] Message delivery receipts (double ticks)

## Notes
- Offline-first architecture (local SQLite)
- Networking/cloud sync to be added later
- WebSocket binds to 0.0.0.0:9001 (accessible on LAN)
- Multi-instance works on same machine via separate Tauri configs
- Custom titlebar requires capabilities/default.json for window permissions in Tauri 2.0
