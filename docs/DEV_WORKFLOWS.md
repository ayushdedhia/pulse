# Pulse - Development Workflows

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

## Running the App (Central Server Architecture)

Pulse uses a central WebSocket server for message relay. All app instances connect to this server.

### Step 1: Start the central server
```bash
cd pulse-server
cargo run
```
You should see: `Pulse server listening on 0.0.0.0:9001`

### Step 2: Start app instances
```bash
# Terminal 2: Start first app instance
npm run tauri dev

# Terminal 3: Start second app instance (optional, for testing)
npm run tauri:test
```

### Architecture
```
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│   App A     │────▶│  Pulse Server   │◀────│   App B     │
│  (Client)   │◀────│  (localhost:9001│────▶│  (Client)   │
└─────────────┘     └─────────────────┘     └─────────────┘
```

- **Server**: Relays messages between clients, tracks online presence
- **Clients**: Connect to `ws://localhost:9001`, send/receive messages
- **Storage**: Each client stores messages locally in SQLite (server doesn't persist)

## Multi-Instance Testing
```bash
# Terminal 1: Start central server
cd pulse-server && cargo run

# Terminal 2: Start main app
npm run tauri dev

# Terminal 3: Start test instance (separate database)
npm run tauri:test
```
Test instance uses separate database and target directory (`target-test/`).

## Features
- Messages sync in real-time between instances via central server
- Deterministic chat IDs (both parties have same chat ID)
- Auto-creates sender as contact when receiving message from unknown user
- Profile modal shows copyable Pulse ID for sharing
- Correct message alignment (own messages on right, others on left)
- Typing indicators filtered to not show self
- Unique default usernames (User + UUID prefix)
- E2E encryption (messages encrypted before leaving client)

## Server Configuration
The server listens on `0.0.0.0:9001` by default. To change:
```bash
PULSE_SERVER_ADDR=0.0.0.0:8080 cargo run
```

The client connects to `ws://localhost:9001` by default. To change (for production):
```bash
PULSE_SERVER_URL=ws://your-server:9001 npm run tauri dev
```
