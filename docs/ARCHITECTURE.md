# Pulse - Architecture

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
│   ├── websocketService.ts    # WebSocket API calls
│   └── cryptoService.ts       # E2E encryption API calls
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
│   ├── manager.rs            # CryptoManager struct with persistent storage
│   ├── storage.rs            # OS Keyring + SQLite key storage
│   └── types.rs              # SerializableKeyPair, EncryptedMessage, IdentityInfo
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
