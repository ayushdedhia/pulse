# Pulse - Architecture

## Project Structure

### Central Server (pulse-server/)
```
pulse-server/
├── Cargo.toml              # Server dependencies
└── src/
    ├── main.rs             # Entry point, TCP listener
    ├── state.rs            # ServerState (connected clients tracking)
    ├── connection.rs       # Per-client WebSocket handler
    └── messages.rs         # WsMessage enum (shared types)
```

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
│   ├── modals/
│   │   ├── NewChatModal.tsx    # Create new chat
│   │   ├── ProfileModal.tsx    # User profile editor
│   │   ├── OnboardingModal.tsx # Phone number onboarding for new users
│   │   └── ContactInfoPanel.tsx # Contact details panel
│   └── ui/
│       └── CountrySelector.tsx # Country code picker with search
├── services/                   # Tauri command wrappers
│   ├── index.ts               # Re-exports all services
│   ├── userService.ts         # User commands
│   ├── chatService.ts         # Chat commands
│   ├── messageService.ts      # Message commands
│   ├── websocketService.ts    # WebSocket commands
│   ├── cryptoService.ts       # E2E encryption commands
│   ├── callService.ts         # Video call management
│   └── turnService.ts         # TURN/ICE server credentials
├── store/                      # Zustand stores
│   ├── chatStore.ts           # Chat list + active chat state
│   ├── messageStore.ts        # Messages by chatId
│   ├── userStore.ts           # Current user state
│   └── uiStore.ts             # UI state (theme, modals)
├── hooks/
│   ├── useWebSocket.ts        # WebSocket connection hook
│   ├── useCrypto.ts           # E2E encryption hook
│   └── useCSSVariable.ts      # Dynamic CSS variable hook for element dimensions
├── data/
│   └── countries.ts           # Country data (240+ countries with dial codes)
├── utils/
│   ├── formatTime.ts          # Time formatting utilities
│   └── cn.ts                  # Tailwind class merging utility
├── context/
│   └── WebSocketContext.tsx   # WebSocket provider (connects to central server)
├── types/
│   └── index.ts               # TypeScript interfaces
├── App.tsx
├── main.tsx
└── index.css                  # Global styles + animations
```

### Backend (src-tauri/src/) - Tauri App
```
src-tauri/src/
├── main.rs                    # Tauri entry point
├── lib.rs                     # App setup + command registration
├── db.rs                      # SQLite setup + seed data
├── models/                    # Data structures
│   ├── mod.rs                # Re-exports
│   ├── user.rs               # User struct
│   ├── chat.rs               # Chat struct
│   ├── message.rs            # Message struct
│   └── input.rs              # Input validation structs
├── commands/                  # IPC handlers by domain
│   ├── mod.rs                # Re-exports all commands
│   ├── user.rs               # get_user, get_current_user, update_user, get_contacts, add_contact
│   ├── chat.rs               # get_chats, create_chat
│   ├── message.rs            # get_messages, send_message, mark_as_read, search_messages, receive_message
│   ├── websocket.rs          # broadcast_message, connect_websocket, broadcast_presence
│   └── turn.rs               # get_turn_credentials (TURN server API)
├── websocket/                 # WebSocket client (connects to central server)
│   ├── mod.rs                # Re-exports + init_websocket
│   ├── client.rs             # WebSocketClient struct
│   └── messages.rs           # WsMessage enum
├── crypto/                    # E2E encryption
│   ├── mod.rs                # Re-exports + Tauri commands
│   ├── manager.rs            # CryptoManager struct with persistent storage
│   ├── storage.rs            # OS Keyring + SQLite key storage
│   └── types.rs              # SerializableKeyPair, EncryptedMessage, IdentityInfo
└── utils/                     # Shared helpers
    ├── mod.rs                # Re-exports
    ├── helpers.rs            # get_self_id(), generate_deterministic_chat_id()
    └── validation.rs         # Input validation utilities
```

## Architecture Overview

### Central Server Model
```
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   App A     │────▶│  Pulse Server   │◀────│   App B     │
│  (Client)   │◀────│  (ws://host:9001│────▶│  (Client)   │
│  SQLite A   │     │   No Storage    │     │  SQLite B   │
└─────────────┘     └─────────────────┘     └─────────────┘
```

- **Server**: Stateless message relay, presence tracking, no persistence
- **Clients**: Full Tauri app with local SQLite, E2E encryption
- **Messages**: Encrypted on client, relayed through server, stored locally

### Message Flow
1. User A types message → Client encrypts → Sends to server
2. Server broadcasts to all connected clients
3. Client B receives → Decrypts → Stores in local SQLite → Updates UI

### Presence Flow
1. Client connects → Sends `Connect { user_id }`
2. Server broadcasts `Presence { is_online: true }` to all clients
3. Client disconnects → Server broadcasts `Presence { is_online: false }`

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
- **messageStore**: `messages{}`, `replyingTo`, `loadMessages()`, `sendMessage()`, `markAsRead()`, `setReplyingTo()`
- **uiStore**: `theme`, `showNewChat`, `showProfile`, etc.

### Backend Module Organization
Commands are organized by domain in separate files:
- `commands::user` - User-related IPC handlers
- `commands::chat` - Chat management handlers
- `commands::message` - Message CRUD handlers
- `commands::websocket` - WebSocket client handlers

Shared utilities extracted to `utils/`:
- `get_self_id(conn)` - Get current user's ID from database
- `generate_deterministic_chat_id(id1, id2)` - Create consistent chat IDs
- Input validation with `garde` crate
