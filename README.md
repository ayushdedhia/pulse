# Pulse

A WhatsApp-like desktop chat application with end-to-end encryption, built with Tauri and React.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)

## Features

- **Real-time Messaging** — WebSocket-based P2P communication
- **End-to-End Encryption** — X25519 key exchange + AES-256-GCM
- **Offline-First** — Local SQLite database
- **LAN Support** — Connect with peers on the same network
- **Dark/Light Theme** — WhatsApp-inspired design

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | React, TypeScript, Tailwind CSS, Zustand |
| Backend | Tauri 2.0, Rust |
| Database | SQLite (rusqlite) |
| Real-time | WebSocket (tokio-tungstenite) |
| Encryption | x25519-dalek, aes-gcm |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (1.70+)
- [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/pulse.git
cd pulse

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Multi-Instance Testing

Test P2P messaging on the same machine:

```bash
# Terminal 1: Main instance (becomes WebSocket server)
npm run tauri dev

# Terminal 2: Test instance (connects as client)
npm run tauri:test
```

## Project Structure

```
pulse/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── services/           # Tauri API abstraction
│   ├── store/              # Zustand state management
│   └── hooks/              # WebSocket & crypto hooks
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # IPC handlers
│   │   ├── models/         # Data structures
│   │   ├── websocket/      # Real-time server
│   │   └── crypto/         # E2E encryption
│   └── Cargo.toml
└── package.json
```

## How It Works

1. First instance starts a WebSocket server on port 9001
2. Additional instances connect as clients
3. Messages are encrypted client-side before transmission
4. Each chat has unique session keys derived via X25519 + HKDF

## License

MIT
