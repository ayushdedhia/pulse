# Pulse - WhatsApp Clone

## Project Overview
A WhatsApp-like desktop chat application built with Tauri (Rust backend) + React (TypeScript frontend).

## Tech Stack
- **Frontend**: React 18 + TypeScript + Tailwind CSS
- **Backend**: Tauri 2.0 (Rust)
- **Build Tool**: Vite
- **State Management**: Zustand
- **Database**: SQLite (local via rusqlite)
- **Real-time**: WebSocket (tokio-tungstenite)
- **Encryption**: X25519 + AES-256-GCM (E2E)
- **Icons**: Lucide React
- **Font**: Inter (Google Fonts)

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

## Project Structure
```
pulse/
├── src/                      # React frontend
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppLayout.tsx    # Main 3-panel layout with resize
│   │   │   ├── Sidebar.tsx      # Navigation sidebar
│   │   │   ├── ChatList.tsx     # Chat list with search/filters
│   │   │   └── ChatWindow.tsx   # Active chat view
│   │   ├── chat/
│   │   │   ├── ChatHeader.tsx   # Chat header with actions
│   │   │   ├── MessageList.tsx  # Scrollable message container
│   │   │   ├── MessageBubble.tsx # Individual message styling
│   │   │   ├── MessageInput.tsx  # Input with emoji/attachments
│   │   │   ├── MessageStatus.tsx # Tick indicators
│   │   │   ├── DateDivider.tsx   # Date separators
│   │   │   └── EmojiPicker.tsx   # Emoji selector grid
│   │   ├── common/
│   │   │   ├── Avatar.tsx       # User avatar with fallback
│   │   │   └── ChatListItem.tsx # Individual chat preview
│   │   └── modals/
│   │       ├── NewChatModal.tsx  # Create new chat
│   │       └── ProfileModal.tsx  # User profile editor
│   ├── hooks/
│   │   ├── useFormatTime.ts     # Time formatting hook
│   │   ├── useWebSocket.ts      # WebSocket connection hook
│   │   └── useCrypto.ts         # E2E encryption hook
│   ├── context/
│   │   └── WebSocketContext.tsx # WebSocket provider
│   ├── store/
│   │   ├── chatStore.ts         # Chat state (Zustand)
│   │   └── uiStore.ts           # UI state (theme, modals)
│   ├── types/
│   │   └── index.ts             # TypeScript interfaces
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css                # Global styles + animations
├── src-tauri/                   # Rust backend
│   ├── src/
│   │   ├── main.rs              # Tauri entry point
│   │   ├── lib.rs               # App setup + command registration
│   │   ├── db.rs                # SQLite setup + seed data
│   │   ├── commands.rs          # IPC command handlers
│   │   ├── websocket.rs         # WebSocket server (port 9001)
│   │   └── crypto.rs            # E2E encryption (X25519 + AES-GCM)
│   ├── icons/                   # App icons (ico, png)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scripts/
│   ├── generate-icons.mjs       # PNG icon generator
│   └── generate-ico.mjs         # ICO file generator
├── .vscode/
│   └── settings.json            # Rust-analyzer config
├── Cargo.toml                   # Workspace config
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
└── tsconfig.json
```

## Design Reference
- Primary green: #00A884 (WhatsApp teal)
- Dark mode background: #111B21
- Dark mode panels: #202C33
- Dark mode hover: #2A3942
- Light mode: #FFFFFF, #F0F2F5
- Font: Inter (Google Fonts)
- Animations: fadeIn, slideUp, scaleIn, typingBounce

## Commands
```bash
npm run tauri dev      # Development
npm run tauri build    # Production build
npm run dev            # Vite dev server only
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
- [ ] Network discovery for LAN peers
- [ ] Message delivery receipts (double ticks)

## Notes
- Offline-first architecture (local SQLite)
- Networking/cloud sync to be added later
- WebSocket currently local only (port 9001)
- Multi-instance works on same machine via separate Tauri configs
