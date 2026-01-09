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

## Multi-Instance Testing
```bash
# Terminal 1: Start main app (becomes WebSocket server)
npm run tauri dev

# Terminal 2: Start test instance (connects as client)
npm run tauri:test
```
Test instance uses separate database and target directory (`target-test/`).

## Multi-Instance P2P Messaging
- First instance becomes WebSocket server (port 9001)
- Second instance auto-connects as client relay
- Messages sync in real-time between instances
- Deterministic chat IDs (both parties have same chat ID)
- Auto-creates sender as contact when receiving message from unknown user
- Profile modal shows copyable Pulse ID for sharing
- Correct message alignment (own messages on right, others on left)
- Typing indicators filtered to not show self
- Unique default usernames (User + UUID prefix)

## Important
After implementing a feature, I will test it thoroughly, after its been tested with no issues/bugs you can finally update this file and give me a good commit message without referencing the claude code in any ways.
