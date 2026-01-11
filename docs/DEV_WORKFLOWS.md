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
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│   App A     │────▶│  Pulse Server    │◀────│   App B     │
│  (Client)   │◀────│ (localhost:9001) │────▶│  (Client)   │
└─────────────┘     └──────────────────┘     └─────────────┘
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

### Environment Variables
| Variable | Used By | Default | Description |
|----------|---------|---------|-------------|
| `PULSE_SERVER_URL` | Rust backend | `ws://localhost:9001` | WebSocket server URL for client connections |
| `VITE_SERVER_URL` | Vite/Frontend | (none) | Frontend server URL (typically mirrors `PULSE_SERVER_URL`) |
| `PULSE_SERVER_ADDR` | Server | `0.0.0.0:9001` | Address the server binds to |
| `PORT` | Server (Railway) | 9001 | Port override (Railway sets this automatically) |

### Running with Local Server (Development)

**Default behavior** - no environment variables needed:

```bash
# Terminal 1: Start local server
cd pulse-server && cargo run
# Output: Pulse server listening on 0.0.0.0:9001

# Terminal 2: Start client (connects to ws://localhost:9001)
npm run tauri dev
```

The client automatically connects to `ws://localhost:9001` when no `PULSE_SERVER_URL` is set.

### Running with Production Server

To test against the production Railway server:

**PowerShell:**
```powershell
$env:PULSE_SERVER_URL="wss://pulse-production-5948.up.railway.app"
$env:VITE_SERVER_URL="wss://pulse-production-5948.up.railway.app"
npm run tauri dev
```

**Bash/Linux/macOS:**
```bash
export PULSE_SERVER_URL=wss://pulse-production-5948.up.railway.app
export VITE_SERVER_URL=wss://pulse-production-5948.up.railway.app
npm run tauri dev
```

**Important:** When using production server:
- Messages are relayed through Railway (internet required)
- You can chat with other users on the production server
- DO NOT run the local server (clients won't connect to it anyway)

### Switching Between Servers

| Scenario | PULSE_SERVER_URL | Local Server |
|----------|------------------|--------------|
| Local development | (unset or `ws://localhost:9001`) | **Run it** |
| Test with production | `wss://pulse-production-5948.up.railway.app` | Don't run |
| LAN testing | `ws://<your-ip>:9001` | **Run it** |

To clear environment variables:

**PowerShell:**
```powershell
Remove-Item Env:PULSE_SERVER_URL -ErrorAction SilentlyContinue
Remove-Item Env:VITE_SERVER_URL -ErrorAction SilentlyContinue
```

**Bash:**
```bash
unset PULSE_SERVER_URL VITE_SERVER_URL
```

## Dev Scripts (PowerShell)

Located in `scripts/`:

### `dev-client.ps1` - Launch Client
```powershell
# Normal launch (connects to default server based on env vars)
.\scripts\dev-client.ps1

# Fresh start (clears database first)
.\scripts\dev-client.ps1 -Fresh

# Second instance (separate data directory)
.\scripts\dev-client.ps1 -Instance 2

# Fresh second instance
.\scripts\dev-client.ps1 -Fresh -Instance 2
```

### `local-test.ps1` - Full Local Test Setup
Starts server + 2 clients with clean databases:
```powershell
# Full setup: cleanup + server + 2 clients
.\scripts\local-test.ps1

# Server only (launch clients manually)
.\scripts\local-test.ps1 -ServerOnly

# Skip cleanup (keep existing data)
.\scripts\local-test.ps1 -SkipCleanup
```

## Production Deployment

### Server (Railway)
The server is deployed to Railway at `wss://pulse-production-5948.up.railway.app`.

Railway auto-deploys on push to main. The server reads the `PORT` env var automatically.

### Client Build
To build the client for production with the Railway server:

**PowerShell:**
```powershell
$env:VITE_SERVER_URL="wss://pulse-production-5948.up.railway.app"
$env:PULSE_SERVER_URL="wss://pulse-production-5948.up.railway.app"
npm run tauri build
```

**Bash:**
```bash
export VITE_SERVER_URL=wss://pulse-production-5948.up.railway.app
export PULSE_SERVER_URL=wss://pulse-production-5948.up.railway.app
npm run tauri build
```

The installers are created in:
- `target/release/bundle/nsis/Pulse_x.x.x_x64-setup.exe`
- `target/release/bundle/msi/Pulse_x.x.x_x64_en-US.msi`

### GitHub Releases
1. Tag the version: `git tag vX.X.X && git push origin vX.X.X`
2. Go to https://github.com/ayushdedhia/pulse/releases
3. Create release from tag and upload the `.exe` installer
