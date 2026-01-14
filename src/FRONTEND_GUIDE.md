# Pulse - Frontend Guide

## Tech Stack
- **Framework**: React 18 + TypeScript
- **Styling**: Tailwind CSS
- **State Management**: Zustand (split stores)
- **Icons**: Lucide React
- **Fonts**: Inter (UI), JetBrains Mono (monospace for IDs/IPs)

## Frontend Structure
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
│   │   └── NetworkModal.tsx    # LAN network settings
│   └── ui/
│       └── CountrySelector.tsx # Country code picker with search
├── services/                   # API abstraction layer
│   ├── index.ts               # Re-exports all services
│   ├── userService.ts         # User API calls
│   ├── chatService.ts         # Chat API calls
│   ├── messageService.ts      # Message API calls
│   ├── websocketService.ts    # WebSocket API calls
│   └── cryptoService.ts       # E2E encryption API calls
├── store/                      # Zustand stores
│   ├── chatStore.ts           # Chat list + active chat state
│   ├── messageStore.ts        # Messages by chatId
│   ├── userStore.ts           # Current user state
│   └── uiStore.ts             # UI state (theme, modals)
├── utils/
│   ├── formatTime.ts          # Time formatting utilities
│   └── cn.ts                  # Tailwind class merging utility (clsx + tailwind-merge)
├── hooks/
│   ├── useWebSocket.ts        # WebSocket connection hook
│   ├── useCrypto.ts           # E2E encryption hook
│   └── useCSSVariable.ts      # Dynamic CSS variable hook for element dimensions
├── data/
│   └── countries.ts           # Country data (240+ countries with dial codes)
├── context/
│   └── WebSocketContext.tsx   # WebSocket provider
├── types/
│   └── index.ts               # TypeScript interfaces
├── App.tsx
├── main.tsx
└── index.css                  # Global styles + animations
```

## Services Layer Pattern
All Tauri API calls are abstracted through services:
```typescript
// Instead of: invoke<User>("get_current_user")
// Use: userService.getCurrentUser()

import { userService, chatService, messageService } from "./services";
```

## Split Zustand Stores
State is split by domain for better separation of concerns:
- **userStore**: `currentUser`, `loadCurrentUser()`, `updateCurrentUser()`
- **chatStore**: `chats[]`, `activeChat`, `loadChats()`, `setActiveChat()`, `createChat()`
- **messageStore**: `messages{}`, `replyingTo`, `loadMessages()`, `sendMessage()`, `markAsRead()`, `setReplyingTo()`
- **uiStore**: `theme`, `showNewChat`, `showProfile`, etc.

## Design Reference
- Primary green: #00A884 (WhatsApp teal)
- Dark mode background: #111B21
- Dark mode panels: #202C33
- Dark mode hover: #2A3942
- Light mode: #FFFFFF, #F0F2F5
- Fonts: Inter (UI), JetBrains Mono (IDs, IPs, phone numbers)
- Animations: fadeIn, slideUp, scaleIn, typingBounce
- Custom titlebar with native window controls (decorations: false)

## UI Rules
- Any OS/network/filesystem work belongs in Rust/Tauri commands, not React.
- Always follow existing patterns (services/, split stores).
- Do NOT invent APIs, commands, types, or file paths.
- If touching Rust commands, update the corresponding frontend service wrapper.
- If adding new commands, update registration + types + service exports.

## Styling Conventions
- **Tailwind classes**: Always write on a single line, never split across multiple lines.
- **Conditional classes**: Use `cn()` utility for dynamic/conditional class merging.

## Custom Hooks

### useCSSVariable
Dynamically set CSS variables based on element dimensions:
```typescript
// Track element height
const ref = useCSSVariable({ variable: 'titlebar-height' });

// Track width
const ref = useCSSVariable({ variable: 'sidebar-width', dimension: 'width' });

// Track both dimensions (sets --name-height and --name-width)
const ref = useCSSVariable({ variable: 'card', dimension: 'both' });
```

## E2E Testing
- Tests located in `e2e/` directory
- Use `?test=onboarding` query param to force onboarding modal for testing
- Page objects in `e2e/fixtures.ts` (OnboardingPage, ChatListPage, ChatPage)
