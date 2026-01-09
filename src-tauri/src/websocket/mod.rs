mod handlers;
mod messages;
mod server;

pub use messages::WsMessage;
pub use server::{NetworkStatus, WebSocketServer};

use rand::RngCore;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

// Global WebSocket server instance
static WS_SERVER: OnceLock<WebSocketServer> = OnceLock::new();

// Global session token for this app instance
static SESSION_TOKEN: OnceLock<String> = OnceLock::new();

// Valid tokens (includes own token + tokens from trusted peers)
static VALID_TOKENS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Generate a new random session token
fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Get the session token for this app instance
pub fn get_session_token() -> &'static str {
    SESSION_TOKEN.get_or_init(|| {
        let token = generate_session_token();
        // Add own token to valid tokens
        get_valid_tokens().lock().unwrap().insert(token.clone());
        token
    })
}

/// Get the set of valid tokens
fn get_valid_tokens() -> &'static Mutex<HashSet<String>> {
    VALID_TOKENS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Validate an authentication token
pub fn validate_token(token: &str) -> bool {
    get_valid_tokens()
        .lock()
        .map(|tokens| tokens.contains(token))
        .unwrap_or(false)
}

/// Add a trusted peer token (e.g., after key exchange)
pub fn add_trusted_token(token: &str) {
    if let Ok(mut tokens) = get_valid_tokens().lock() {
        tokens.insert(token.to_string());
    }
}

pub fn get_ws_server() -> &'static WebSocketServer {
    WS_SERVER.get_or_init(WebSocketServer::new)
}

pub async fn init_websocket_server() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    // Ensure token is generated before server starts
    let _ = get_session_token();
    let server = get_ws_server();
    server.start("127.0.0.1:9001").await
}
